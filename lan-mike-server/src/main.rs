use std::thread;
use std::sync::{Arc, Mutex};
use std::mem;
use std::net::UdpSocket;

use cpal::{StreamData, UnknownTypeOutputBuffer, EventLoop};

const EVENT_JOIN: u8 = 2;
const EVENT_SPEAK: u8 = 4;
const EVENT_LEAVE: u8 = 8;

fn main() {
  let socket = UdpSocket::bind("127.0.0.1:34222").expect("Could not listen on port 34254");
  let mut buffer = vec![0; 1024];

  let event_loop = EventLoop::new();
  let output_device = cpal::default_output_device().expect("no output device available");
  let mut supported_output_formats = output_device.supported_output_formats().expect("error while querying formats");
  let format = supported_output_formats.next().expect("no supported formats?!").with_max_sample_rate();
  let output_stream = event_loop.build_output_stream(&output_device, &format).unwrap();
  event_loop.play_stream(output_stream);

  let output = Arc::new(Mutex::new(vec![]));
  let output_reader = Arc::clone(&output);

  thread::spawn(move || {
    event_loop.run(|_stream_id, stream_data| {
      match stream_data {
        StreamData::Output { buffer: UnknownTypeOutputBuffer::F32(mut buffer) } => {
          let mut output = output_reader.lock().unwrap();
          let length = output.len();
          if length >= 1024 {
            output.reverse();
            let mut samples = output.split_off(length - 1024);
            output.reverse();
            samples.reverse();
            for elem in buffer.iter_mut() {
              *elem = samples.pop().unwrap();
            }
          } else {
            for elem in buffer.iter_mut() {
              *elem = 0.0;
            }
          }
        },
        _ => (),
      }
    });
  });

  loop {
    let (_amt, _src) = socket.recv_from(&mut buffer).unwrap();
    for i in 0..256 {
      let bytes = [buffer[i*4], buffer[i*4 + 1], buffer[i*4 + 2], buffer[i*4 + 3]];
      let sample: f32 = unsafe { mem::transmute(bytes) };
      output.lock().unwrap().push(sample);
    }
    // match buffer[0] {
    //   // EVENT_JOIN => { println!("{} has joined!", src.ip()); },
    //   EVENT_SPEAK => {

    //   },
    //   // EVENT_LEAVE => { println!("{} has left!", src.ip()); },
    //   _ => {},
    // }

  }
}
