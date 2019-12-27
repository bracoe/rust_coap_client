use std::net::{SocketAddr, UdpSocket};
use std::thread;

//declare a structure
struct CoapMessageHeader {
	version:u8,
	coap_type:u8,
	token_length:u8,
	coap_class:u8,
	code:u8,
	id:u16,
	token:u64
}

impl std::fmt::Display for CoapMessageHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(version: {:#x}, type: {:#x} , token length: {:#x}, class: {:#x}, code: {:#x}, id: {:#x}, token: {:#x})", self.version, self.coap_type, self.token_length, self.coap_class, self.code, self.id, self.token)
    }
}

fn main() {
    println!("Hello, world!");

	let socket = UdpSocket::bind("127.0.0.1:5683").expect("couldn't bind to address");
	assert_eq!(socket.peer_addr().unwrap_err().kind(), std::io::ErrorKind::NotConnected);
	
	loop{
		let mut message_buffer = [0u8; 1500]; //MTU assumed to be 1500
		let socket_clone = socket.try_clone().expect("Failed to clone socket");
		match socket.recv_from(&mut message_buffer){
			Ok((_, sender_adress)) => {
				thread::spawn(move || {
					handle_request(socket_clone, message_buffer, sender_adress);
				});
			},
			Err(e) => {
				eprintln!("Error while receiving datagram: {}", e);
			}
		}
	}
}

fn handle_request(_socket: UdpSocket, message_buffer: [u8; 1500], _sender_adress: SocketAddr){
	println!("We got a message!");
	
	print_udp_datagram_buffer(message_buffer);
	parse_coap_header(message_buffer);
}

fn parse_coap_header(message_buffer: [u8; 1500]){
	
	//TODO: Check that we actually have values in the array
	
	//Get id
	let id_upper_nibble = (message_buffer[2] as u16) << 8;
	let id_lower_nibble = message_buffer[3] as u16;
	
	let token_num_of_bytes = 0x0Fu8 & message_buffer[0] as u8;
	
	let coap_token;
	
	match token_num_of_bytes {
		1 => coap_token = message_buffer[4] as u64,
		2 => coap_token = ((message_buffer[4] as u64) << 8) | (message_buffer[5] as u64),
		3 => coap_token = ((message_buffer[4] as u64) << 16) | ((message_buffer[5] as u64) << 8) | (message_buffer[6] as u64),
		4 => coap_token = ((message_buffer[4] as u64) << 32) | ((message_buffer[5] as u64) << 16) | ((message_buffer[6] as u64) << 8) | (message_buffer[7] as u64),
		_ => coap_token = 0,
	}
	
	let coap_header = CoapMessageHeader {
		version:(0xC0u8 & message_buffer[0])>>6,
		coap_type:(0x30u8 & message_buffer[0])>>4,
		token_length:token_num_of_bytes,
		coap_class:(0x0Fu8 & message_buffer[0]),
		code:(message_buffer[1]) & 0x1Fu8,
		id:id_upper_nibble|id_lower_nibble,
		token:coap_token
	};
	
	println!("Used Display: {}", coap_header);
	
}

fn print_udp_datagram_buffer( message_buffer: [u8; 1500]){
	
	
	let mut i = 0;
	while message_buffer[i] != 0 {
    	println!("{:#x}", message_buffer[i]);
		i += 1;
	}
}