use crate::house::room::Room;
use crate::house::traits::{DeviceVisitor, SmartDevice};
use std::collections::HashMap;
use std::io::{Error, ErrorKind};

use super::errors::HouseError;

#[derive(Default, Debug)]
pub struct House {
    pub name: String,
    rooms: HashMap<String, Room>,
}

impl House {
    pub fn add_room(&mut self, room_id: &str) -> Result<(), HouseError> {
        if room_id.is_empty() {
            return Err(HouseError::EmptyRoomName());
        }
        if self.rooms.contains_key(room_id) {
            return Err(HouseError::RoomAlreadyExists(room_id.to_string()));
        }
        self.rooms.insert(room_id.to_string(), Room::new(room_id));
        Ok(())
    }

    pub fn del_room(&mut self, room_id: &str) -> Result<(), HouseError> {
        match self.rooms.get(room_id) {
            Some(room) => match room.devices.len() {
                0 => Err(HouseError::NonEmptyRoomRemoving(room_id.to_string())),
                _ => {
                    self.rooms.remove(room_id);
                    Ok(())
                }
            },
            None => Err(HouseError::RoomNotFound(room_id.to_string())),
        }
    }

    pub fn get_room_ids(&self) -> Vec<&String> {
        Vec::from_iter(self.rooms.keys())
    }

    pub fn add_device<T: SmartDevice + 'static>(
        &mut self,
        room_id: &str,
        device: T,
    ) -> Result<(), HouseError> {
        match self.rooms.get_mut(room_id) {
            Some(room) => room.add_device(device),
            None => Err(HouseError::RoomAlreadyExists(room_id.to_string())),
        }
    }

    pub fn visit_devices_mut<T: DeviceVisitor>(
        &mut self,
        visitor: &mut T,
        room_id: Option<&str>,
    ) -> Result<(), Error> {
        match room_id {
            Some(name) => match self.rooms.get_mut(name) {
                Some(room) => room.visit_devices_mut(visitor),
                None => {
                    return Err(Error::new(
                        ErrorKind::NotFound,
                        "Room with such id doesn't exist",
                    ))
                }
            },
            None => {
                for r in self.rooms.values_mut() {
                    r.visit_devices_mut(visitor);
                }
                return Ok(());
            }
        }
        Ok(())
    }

    pub fn visit_devices<T: DeviceVisitor>(
        &self,
        visitor: &mut T,
        room_id: Option<&str>,
    ) -> Result<(), Error> {
        match room_id {
            Some(name) => match self.rooms.get(name) {
                Some(room) => room.visit_devices(visitor),
                None => {
                    return Err(Error::new(
                        ErrorKind::NotFound,
                        "Room with such id doesn't exist",
                    ))
                }
            },
            None => {
                for r in self.rooms.values() {
                    r.visit_devices(visitor);
                }
                return Ok(());
            }
        }
        Ok(())
    }

    pub fn extract_device<T: 'static>(
        &mut self,
        room_id: &str,
        device_id: &str,
    ) -> Result<T, HouseError> {
        match self.rooms.get_mut(room_id) {
            Some(room) => room.extract_device(device_id),
            None => Err(HouseError::RoomNotFound(room_id.to_string())),
        }
    }
}
