use std::mem;
use std::thread;
use std::net::{UdpSocket};

use cpal::{StreamData, UnknownTypeOutputBuffer, UnknownTypeInputBuffer, EventLoop};

const EVENT_JOIN: u8 = 2;
const EVENT_SPEAK: u8 = 4;
const EVENT_LEAVE: u8 = 8;

fn main() {
  let socket = UdpSocket::bind("127.0.0.1:34111").expect("Could not listen on port 34111");
  socket.connect("127.0.0.1:34222").expect("connect function failed");

  let event_loop = EventLoop::new();
  let output_device = cpal::default_output_device().expect("no output device available");
  let input_device = cpal::default_input_device().expect("no input device available");
  let mut supported_output_formats = output_device.supported_output_formats().expect("error while querying formats");
  let format = supported_output_formats.next().expect("no supported formats?!").with_max_sample_rate();
  let output_stream = event_loop.build_output_stream(&output_device, &format).unwrap();
  let input_stream = event_loop.build_input_stream(&input_device, &format).unwrap();
  event_loop.play_stream(output_stream);
  event_loop.play_stream(input_stream);

  // let mut input = vec![];
  // let mut output = vec![];

  let recv_socket = socket.try_clone().unwrap();
  thread::spawn(move || {
    let mut buffer = vec![0; 1024];
    loop {
      recv_socket.recv(&mut buffer).unwrap();
    }
  });

  let send_socket = socket.try_clone().unwrap();
  event_loop.run(move |_stream_id, stream_data| {
    match stream_data {
      StreamData::Input { buffer: UnknownTypeInputBuffer::F32(buffer) } => {
        let buffer = buffer.as_ref().to_vec();
        let mut msg1 = vec![];
        for sample in &buffer[0..256] {
          let bytes: [u8; 4] = unsafe { mem::transmute(*sample) };
          for byte in bytes.iter() { msg1.push(*byte); }
        }
        send_socket.send(&msg1).unwrap();
        let mut msg2 = vec![];
        for sample in &buffer[256..512] {
          let bytes: [u8; 4] = unsafe { mem::transmute(*sample) };
          for byte in bytes.iter() { msg2.push(*byte); }
        }
        send_socket.send(&msg2).unwrap();
        let mut msg3 = vec![];
        for sample in &buffer[512..768] {
          let bytes: [u8; 4] = unsafe { mem::transmute(*sample) };
          for byte in bytes.iter() { msg3.push(*byte); }
        }
        send_socket.send(&msg3).unwrap();
        let mut msg4 = vec![];
        for sample in &buffer[768..1024] {
          let bytes: [u8; 4] = unsafe { mem::transmute(*sample) };
          for byte in bytes.iter() { msg4.push(*byte); }
        }
        send_socket.send(&msg4).unwrap();
      },
      StreamData::Output { buffer: UnknownTypeOutputBuffer::F32(mut buffer) } => {
        for elem in buffer.iter_mut() {
          *elem = 0.0;
        }
      },
      _ => (),
    }
  });

  socket.send(&[EVENT_LEAVE]).unwrap();
}
