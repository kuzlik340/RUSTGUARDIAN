use crate::push_log;

#[derive(Clone)]
// Represents a single USB device with a unique ID and its name
pub struct DeviceEntry {
    pub id: String,
    pub name: String,
}

pub struct DeviceList {
    // Fixed-size list of devices, allowing None for empty slots
    devices: Vec<Option<DeviceEntry>>,
}

impl DeviceList {
    // Creates a new DeviceList with a given capacity
    pub fn new(size: usize) -> Self {
        DeviceList {
            devices: vec![None; size],
        }
    }

    // Logs all non-empty device entries to the log system
    pub fn log_devices(&self) {
        push_log("[REVIEW DEVICE LIST] ---------------------".to_string());
        let mut any_found = false;
        for (i, entry) in self.devices.iter().enumerate() {
            if let Some(device) = entry {
                push_log(format!("{}: [{}] {}", i, device.id, device.name));
                any_found = true;
            }
        }

        if !any_found {
            push_log("[REVIEW DEVICE LIST] is empty".to_string());
        }
    }


    // Adds a new device to the first available empty slot
    // Rejects duplicates based on device ID
    pub fn add_device(&mut self, device: DeviceEntry) -> Result<usize, String> {
        if self.contains_id(&device.id) {
            return Err(format!("Device with ID '{}' already exists", device.id));
        }

        if let Some(index) = self.devices.iter().position(|d| d.is_none()) {
            self.devices[index] = Some(device);
            Ok(index)
        } else {
            Err("No empty slots available".to_string())
        }
    }

    // Removes a device by index, setting its slot to None
    pub fn remove_device(&mut self, index: usize) -> Result<(), String> {
        if index < self.devices.len() {
            self.devices[index] = None;
            Ok(())
        } else {
            Err("Invalid index".to_string())
        }
    }

    // Returns a reference to a device entry by index, if it exists
    pub fn get(&self, index: usize) -> Option<&DeviceEntry> {
        self.devices.get(index).and_then(|opt| opt.as_ref())
    }

    // Checks whether a device with the given ID is already present
    pub fn contains_id(&self, id: &str) -> bool {
        self.devices.iter().flatten().any(|d| d.id == id)
    }
}
