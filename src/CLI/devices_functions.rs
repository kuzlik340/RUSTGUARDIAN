#[derive(Clone)]
pub struct DeviceEntry {
    pub id: String,
    pub name: String,
}

pub struct DeviceList {
    devices: Vec<Option<DeviceEntry>>,
}

impl DeviceList {
    pub fn new(size: usize) -> Self {
        DeviceList {
            devices: vec![None; size],
        }
    }

    pub fn log_devices(&self) {
        push_log("[DEVICE LIST] ---------------------".to_string());
        for (i, entry) in self.devices.iter().enumerate() {
            match entry {
                Some(device) => {
                    push_log(format!("{}: [{}] {}", i, device.id, device.name));
                }
                None => {
                    push_log(format!("{}: <empty>", i));
                }
            }
        }
    }

    pub fn add_device(&mut self, device: DeviceEntry) -> Result<usize, String> {
        if let Some(index) = self.devices.iter().position(|d| d.is_none()) {
            self.devices[index] = Some(device);
            Ok(index)
        } else {
            Err("No empty slots available".to_string())
        }
    }

    pub fn remove_device(&mut self, index: usize) -> Result<(), String> {
        if index < self.devices.len() {
            self.devices[index] = None;
            Ok(())
        } else {
            Err("Invalid index".to_string())
        }
    }

    pub fn get(&self, index: usize) -> Option<&DeviceEntry> {
        self.devices.get(index).and_then(|opt| opt.as_ref())
    }

    pub fn contains_id(&self, id: &str) -> bool {
        self.devices.iter().flatten().any(|d| d.id == id)
    }
}
