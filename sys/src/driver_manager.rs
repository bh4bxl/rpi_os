use crate::{
    info,
    synchronization::{interface::Mutex, NullLock},
};

const NUM_DRIVERS: usize = 5;

/// Driver interfaces.
pub mod interface {
    /// Device Driver functions.
    pub trait DeviceDriver {
        /// Return a compatibility string for identifying the driver.
        fn compatible(&self) -> &'static str;

        /// Called by the kernel to bring up the device.
        /// # Safety
        unsafe fn init(&self) -> Result<(), &'static str> {
            Ok(())
        }
    }
}

/// Tpye to be used as an optional callback after a driver's init() has run.
pub type DeviceDriverPostInitCallback = unsafe fn() -> Result<(), &'static str>;

/// A descriptor for device drivers.
#[derive(Clone, Copy)]
pub struct DeviceDriverDescriptor {
    device_driver: &'static (dyn interface::DeviceDriver + Sync),
    post_init_callback: Option<DeviceDriverPostInitCallback>,
}

impl DeviceDriverDescriptor {
    /// Create an instance.
    pub fn new(
        device_driver: &'static (dyn interface::DeviceDriver + Sync),
        post_init_callback: Option<DeviceDriverPostInitCallback>,
    ) -> Self {
        Self {
            device_driver,
            post_init_callback,
        }
    }
}

struct DriverManagerInner {
    next_index: usize,
    descriptors: [Option<DeviceDriverDescriptor>; NUM_DRIVERS],
}

impl DriverManagerInner {
    pub const fn new() -> Self {
        Self {
            next_index: 0,
            descriptors: [None; NUM_DRIVERS],
        }
    }
}

/// Provides device driver management functions.
pub struct DriverManager {
    inner: NullLock<DriverManagerInner>,
}

impl DriverManager {
    /// Create an instance.
    pub const fn new() -> Self {
        Self {
            inner: NullLock::new(DriverManagerInner::new()),
        }
    }

    /// Register a device driver with the kernel.
    pub fn register_driver(&self, descriptor: DeviceDriverDescriptor) {
        self.inner.lock(|inner| {
            inner.descriptors[inner.next_index] = Some(descriptor);
            inner.next_index += 1;
        })
    }

    /// Helper for iterating over registered drivers.
    pub fn for_each_descriptor<'a>(&'a self, f: impl FnMut(&'a DeviceDriverDescriptor)) {
        self.inner.lock(|inner| {
            inner
                .descriptors
                .iter()
                .filter_map(|x| x.as_ref())
                .for_each(f)
        })
    }

    /// Fully initialize all drivers.
    /// # Safety
    pub unsafe fn init_drivers(&self) {
        self.for_each_descriptor(|descriptor| {
            // Initialize driver
            if let Err(x) = descriptor.device_driver.init() {
                panic!(
                    "Error initializing driver: {}: {}",
                    descriptor.device_driver.compatible(),
                    x
                );
            }

            // Call corresponding post init callback
            if let Some(callback) = &descriptor.post_init_callback {
                if let Err(x) = callback() {
                    panic!(
                        "Error during driver post-init callback: {}: {}",
                        descriptor.device_driver.compatible(),
                        x
                    );
                }
            }
        });
    }

    /// Enumerate all registered device drivers.
    pub fn enumerate(&self) {
        let mut i: usize = 1;
        self.for_each_descriptor(|descriptor| {
            info!("        {}. {}", i, descriptor.device_driver.compatible());
            i += 1;
        });
    }
}

static DRIVER_MANAGER: DriverManager = DriverManager::new();

pub fn driver_manager() -> &'static DriverManager {
    &DRIVER_MANAGER
}
