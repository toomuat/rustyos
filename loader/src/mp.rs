use log::info;
use uefi::prelude::*;
use uefi::proto::pi::mp::MpServices;

// https://github.com/rust-osdev/uefi-rs/blob/master/uefi-test-runner/src/proto/pi/mp.rs
// 第1章　ブートローダーを変更し各APをカーネルへジャンプさせる yuma.ohgami.jp/x86_64-Jisaku-OS-5/01_boot.html

/// Number of cores qemu is configured to have
const NUM_CPUS: usize = 4;

pub struct SystemConf<'a> {
    pub kernel_entry_func: extern "sysv64" fn(
        fb: *mut common::graphics::FrameBuffer,
        mi: *mut uefi::proto::console::gop::ModeInfo,
        rsdp: u64,
        proc_number: usize,
    ) -> (),
    pub proc_number: usize,
    pub mps: &'a mut MpServices,
}

impl<'a> SystemConf<'a> {
    pub fn kernel_entry(
        &self,
        fb: *mut common::graphics::FrameBuffer,
        mi: *mut uefi::proto::console::gop::ModeInfo,
        rsdp: u64,
        proc_number: usize,
    ) {
        (self.kernel_entry_func)(fb, mi, rsdp, proc_number);
    }
}

pub fn multi_processor_test(
    bt: &BootServices,
    kernel_entry: extern "sysv64" fn(
        fb: *mut common::graphics::FrameBuffer,
        mi: *mut uefi::proto::console::gop::ModeInfo,
        rsdp: u64,
        proc_number: usize,
    ) -> (),
) {
    info!("multi_processor_test");

    if let Ok(mp_support) = bt.locate_protocol::<MpServices>() {
        let mp_support = mp_support
            .expect("Warnings encountered while opening multi-processor services protocol");
        let mp_support = unsafe { &mut *mp_support.get() };

        let mut system_conf = SystemConf {
            kernel_entry_func: kernel_entry,
            proc_number: 0,
            mps: mp_support,
        };

        // bt.stall(3000);

        // info!("test_get_number_of_processors");
        // test_get_number_of_processors(mp_support);
        // info!("test_get_processor_info");
        // test_get_processor_info(mp_support);
        // info!("test_startup_all_aps");
        // system_conf.mps = *mp_support;
        test_startup_all_aps(bt, system_conf);
        // info!("test_startup_this_ap");
        // test_startup_this_ap(mp_support, bt, system_conf);
        // info!("test_enable_disable_ap");
        // test_enable_disable_ap(mp_support);
        // info!("test_switch_bsp_and_who_am_i");
        // test_switch_bsp_and_who_am_i(mp_support);
        info!("mp test end");
    } else {
        info!("Multi-processor services protocol is not supported");
    }
}

fn test_startup_all_aps(bt: &BootServices, system_conf: SystemConf) {
    // Ensure that APs start up
    // let counter = AtomicUsize::new(0);
    // let counter_ptr: *mut c_void = &counter as *const _ as *mut _;
    // bt.stall(1_000);
    // let mps_ptr: *mut c_void = mps as *const _ as *mut _;
    let sysconf_ptr: *mut c_void = &system_conf as *const _ as *mut _;
    system_conf
        .mps
        .startup_all_aps(false, jump_kernel, sysconf_ptr, None)
        .unwrap()
        .unwrap();
    // mps.startup_all_aps(false, jump_kernel, mps_ptr, None)
    //     .unwrap()
    //     .unwrap();
    // mps.startup_all_aps(false, proc_increment_atomic, counter_ptr, None)
    //     .unwrap()
    //     .unwrap();
    // assert_eq!(counter.load(Ordering::Relaxed), NUM_CPUS - 1);

    // Make sure that timeout works
    let bt_ptr: *mut c_void = bt as *const _ as *mut _;
    let ret = system_conf.mps.startup_all_aps(
        false,
        proc_wait_100ms,
        bt_ptr,
        Some(Duration::from_millis(50)),
    );
    assert_eq!(ret.map_err(|err| err.status()), Err(Status::TIMEOUT));
}

extern "efiapi" fn jump_kernel(arg: *mut c_void) {
    // todo!()

    let system_conf: &SystemConf = unsafe { &*(arg as *const _) };
    let proc_number = system_conf.mps.who_am_i().unwrap().unwrap();

    system_conf.kernel_entry(
        0 as *mut common::graphics::FrameBuffer,
        0 as *mut uefi::proto::console::gop::ModeInfo,
        0,
        proc_number,
    );

    // kernel_entry(
    //     &mut fb as *mut FrameBuffer,
    //     &mut mi as *mut uefi::proto::console::gop::ModeInfo,
    //     rsdp,
    //     proc_number,
    // );
}

fn test_get_number_of_processors(mps: &MpServices) {
    let proc_count = mps.get_number_of_processors().unwrap().unwrap();

    // Ensure we can see all of the requested CPUs
    assert_eq!(proc_count.total, NUM_CPUS);

    // All CPUs should be enabled
    assert_eq!(proc_count.total, proc_count.enabled);
}

fn test_get_processor_info(mps: &MpServices) {
    // Disable second CPU for this test
    mps.enable_disable_ap(1, false, None).unwrap().unwrap();

    // Retrieve processor information from each CPU
    let cpu0 = mps.get_processor_info(0).unwrap().unwrap();
    let cpu1 = mps.get_processor_info(1).unwrap().unwrap();
    let cpu2 = mps.get_processor_info(2).unwrap().unwrap();

    // Check that processor_id fields are sane
    assert_eq!(cpu0.processor_id, 0);
    assert_eq!(cpu1.processor_id, 1);
    assert_eq!(cpu2.processor_id, 2);

    // Check that only CPU 0 is BSP
    assert!(cpu0.is_bsp());
    assert!(!cpu1.is_bsp());
    assert!(!cpu2.is_bsp());

    // Check that only the second CPU is disabled
    assert!(cpu0.is_enabled());
    assert!(!cpu1.is_enabled());
    assert!(cpu2.is_enabled());

    // Enable second CPU back
    mps.enable_disable_ap(1, true, None).unwrap().unwrap();
}

use core::ffi::c_void;
// use core::marker::StructuralEq;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::time::Duration;
use uefi::Status;

extern "efiapi" fn proc_increment_atomic(arg: *mut c_void) {
    let counter: &AtomicUsize = unsafe { &*(arg as *const _) };
    counter.fetch_add(1, Ordering::Relaxed);

    // info!("*");
}
extern "efiapi" fn proc_increment_atomic2(arg: *mut c_void) {
    let counter: &AtomicUsize = unsafe { &*(arg as *const _) };
    counter.fetch_add(1, Ordering::Relaxed);

    // info!("*");
}
extern "efiapi" fn proc_increment_atomic3(arg: *mut c_void) {
    let counter: &AtomicUsize = unsafe { &*(arg as *const _) };
    // let counter2: &AtomicUsize = unsafe { &(*(arg as *const AtomicUsize).offset(1)) };
    counter.fetch_add(1, Ordering::Relaxed);
    // counter2.fetch_add(1, Ordering::Relaxed);

    // info!("*");
}

// Checking timeout
extern "efiapi" fn proc_wait_100ms(arg: *mut c_void) {
    let bt: &BootServices = unsafe { &*(arg as *const _) };
    bt.stall(100_000);

    // dont pass
    // info!("*");
    // loop {}
}

fn test_startup_this_ap(mps: &MpServices, bt: &BootServices, mut system_conf: SystemConf) {
    // Ensure that each AP starts up
    let counter = AtomicUsize::new(0);
    let counter2 = AtomicUsize::new(0);
    let counter_ptr: *mut c_void = &counter as *const _ as *mut _;
    // let sysconf_ptr: *mut c_void = &system_conf as *const _ as *mut _;
    let t = mps.who_am_i().unwrap().unwrap();
    info!("mps.who_am_i().unwrap().unwrap() {}", t);
    for i in 1..NUM_CPUS {
        bt.stall(6_000); // to avoid conflict ?
        system_conf.proc_number = mps.who_am_i().unwrap().unwrap();
        let sysconf_ptr: *mut c_void = &system_conf as *const _ as *mut _;
        // mps.startup_this_ap(i, jump_kernel, sysconf_ptr, None)
        //     .unwrap()
        //     .unwrap();
        // mps.startup_this_ap(i, proc_increment_atomic2, counter_ptr, None)
        //     .unwrap()
        //     .unwrap();
    }
    // assert_eq!(
    //     (*counter_ptr_list_ptr[0]).load(Ordering::Relaxed),
    //     NUM_CPUS - 1
    // );
    // assert_eq!(counter.load(Ordering::Relaxed), NUM_CPUS - 1);
    // assert_eq!(counter2.load(Ordering::Relaxed), NUM_CPUS - 1);

    // Make sure that timeout works for each AP
    let bt_ptr: *mut c_void = bt as *const _ as *mut _;
    for i in 1..NUM_CPUS {
        let ret = mps.startup_this_ap(i, proc_wait_100ms, bt_ptr, Some(Duration::from_millis(50)));
        assert_eq!(ret.map_err(|err| err.status()), Err(Status::TIMEOUT));
    }
}

fn test_enable_disable_ap(mps: &MpServices) {
    // Disable second CPU
    mps.enable_disable_ap(1, false, None).unwrap().unwrap();

    // Ensure that one CPUs is disabled
    let proc_count = mps.get_number_of_processors().unwrap().unwrap();
    assert_eq!(proc_count.total - proc_count.enabled, 1);

    // Enable second CPU back
    mps.enable_disable_ap(1, true, None).unwrap().unwrap();

    // Ensure that all CPUs are enabled
    let proc_count = mps.get_number_of_processors().unwrap().unwrap();
    assert_eq!(proc_count.total, proc_count.enabled);

    // Mark second CPU as unhealthy and check it's status
    mps.enable_disable_ap(1, true, Some(false))
        .unwrap()
        .unwrap();
    let cpu1 = mps.get_processor_info(1).unwrap().unwrap();
    assert!(!cpu1.is_healthy());

    // Mark second CPU as healthy again and check it's status
    mps.enable_disable_ap(1, true, Some(true)).unwrap().unwrap();
    let cpu1 = mps.get_processor_info(1).unwrap().unwrap();
    assert!(cpu1.is_healthy());
}

fn test_switch_bsp_and_who_am_i(mps: &MpServices) {
    // Normally BSP starts on on CPU 0
    let proc_number = mps.who_am_i().unwrap().unwrap();
    info!("a");
    assert_eq!(proc_number, 0);

    // Do a BSP switch
    mps.switch_bsp(1, true).unwrap().unwrap();

    // We now should be on CPU 1
    let proc_number = mps.who_am_i().unwrap().unwrap();
    info!("b");
    assert_eq!(proc_number, 1);

    // Switch back
    info!("c");
    // panic here
    mps.switch_bsp(0, true).unwrap().unwrap();

    // We now should be on CPU 0 again
    info!("d");
    let proc_number = mps.who_am_i().unwrap().unwrap();
    info!("proc_number: {}", proc_number);
    assert_eq!(proc_number, 0);
}
