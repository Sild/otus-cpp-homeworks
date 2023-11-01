use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use rand::Rng;

use crate::devices::IdType;
use crate::house::traits::SmartDevice;

pub struct Thermo {
    id: IdType,
    pub is_on: bool,
    cur_temp: f32,
    threads: Vec<JoinHandle<()>>,
    report_stop: Arc<AtomicBool>,
}

impl SmartDevice for Thermo {
    fn get_id(&self) -> &str {
        &self.id
    }
}

impl Thermo {
    pub fn new(id: String) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            id,
            is_on: false,
            cur_temp: rng.gen::<f32>(),
            threads: Vec::default(),
            report_stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn get_report(&self) -> String {
        format!(
            "type: thermometer, id: {}, is_on: {}, cur_temp: {}",
            self.id, self.is_on, self.cur_temp
        )
    }

    pub fn enable_udp_updates(&mut self, in_port: u64, out_port: u64) {
        let stop_flag = self.report_stop.clone();
        let cur_temp_send = Arc::new(Mutex::new(self.cur_temp));
        let cur_temp_rcv = cur_temp_send.clone();

        let send_th = thread::spawn(move || {
            let out_addr = format!("127.0.0.1:{}", out_port);
            let socket = UdpSocket::bind("127.0.0.1:0").unwrap();

            while !stop_flag.load(Ordering::Relaxed) {
                let buf = format!("cur_temp: {}\n", cur_temp_send.lock().unwrap()).into_bytes();
                socket.send_to(buf.as_slice(), &out_addr.as_str()).unwrap();
                println!("report is sent");
                thread::sleep(Duration::from_secs(1));
            }
            println!("sending thread stopped");
        });

        let stop_flag = self.report_stop.clone();
        let rcv_th = thread::spawn(move || {
            let in_addr = format!("127.0.0.1:{}", in_port);
            let socket = UdpSocket::bind(in_addr).unwrap();

            while !stop_flag.load(Ordering::Relaxed) {
                let mut input_buf = [0; 10];
                let (r_size, _) = socket.recv_from(&mut input_buf).unwrap();
                if r_size > 0 {
                    let str = String::from_utf8(input_buf[..r_size].to_vec()).unwrap();
                    match str.parse::<f32>() {
                        Ok(new_val) => {
                            *cur_temp_rcv.lock().unwrap() = new_val;
                            println!("new value received: {}", new_val);
                        }
                        Err(e) => {
                            println!("fail to parse input to f32: {:?}", e)
                        }
                    }
                }
            }
            println!("receiving thread stopped");
        });

        self.threads.push(send_th);
        self.threads.push(rcv_th);
    }
}

// app doesn't wait for drop =(
// impl Drop for Thermo {
//     fn drop(&mut self) {
//         println!("thermo drop called");
//         self.report_stop.store(true, Ordering::Relaxed);
//         match self.report_th.take() {
//             Some(th) => { let _ = th.join(); }
//             None => {}
//         }
//     }
// }
