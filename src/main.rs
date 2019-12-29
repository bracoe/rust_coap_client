use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::collections::LinkedList;

const MAX_MTU: usize = 15000; //MTU assumed to be 1500

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

struct CoapMessage{
	header:CoapMessageHeader,
	option_list:LinkedList<Option>,
	payload: Vec<u8> 
}

struct Option{
	number:u16,
	length:u16,
	value: Vec<u8>
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
		let mut udp_buffer = [0u8; MAX_MTU]; 
		let socket_clone = socket.try_clone().expect("Failed to clone socket");
		match socket.recv_from(&mut udp_buffer){
			Ok((_, sender_adress)) => {
				thread::spawn(move || {
					handle_request(socket_clone, udp_buffer, sender_adress);
				});
			},
			Err(e) => {
				eprintln!("Error while receiving datagram: {}", e);
			}
		}
	}
}

fn handle_request(_socket: UdpSocket, udp_buffer: [u8; MAX_MTU], _sender_adress: SocketAddr){
	//Get the coap message
	let message = parse_message_from_datagram(udp_buffer);
	
	//execute message request
	
}

fn execute_request(message:CoapMessage) -> Result<CoapMessage, &'static str>{
	
	match message.header.coap_class {
		
		0 => { //method
			match message.header.code {
				0 => {
					println!("Got method code: {}", message.header.code);
				}
				1 => {
					println!("Got method code: {}", message.header.code);
				}
				2 => {
					println!("Got method code: {}", message.header.code);
				}
				3 => {
					println!("Got method code: {}", message.header.code);
				}
				4 => {
					println!("Got method code: {}", message.header.code);
				}
				_ => {
					println!("Got method code: {}", message.header.code);
				}
			}
		}
		
		2 => { //success
			println!("Got success code: {}", message.header.code);
		}
		
		4 => {
			println!("Got client error code: {}", message.header.code);
		}
		
		5 => {
			println!("Got server error code: {}", message.header.code);
		}
		
		7 => {
			println!("Got signal code: {}", message.header.code);
		}
		
		_ => {
			Err("Got code that is not defined!");
		}
	}
	
	Ok()
}

fn handle_get_request(message:CoapMessage) -> Result<u8, &'static str>{
	assert_eq!(message.header.coap_class, 0); //make sure the message is a method
	assert_eq!(message.header.code, 1); //make sure the message is a GET
	
	assert!(!message.option_list.is_empty());
	
	
}

fn parse_message_from_datagram(udp_buffer: [u8; MAX_MTU]) -> CoapMessage {
	
	println!("We got a message!");
	
	print_udp_datagram_buffer(udp_buffer);
	
	let message_header = parse_coap_header(udp_buffer);
	assert!(message_header.is_ok());
	let  message_header = message_header.unwrap();
	
	let options_list = parse_coap_options(message_header.token_length,udp_buffer);
	assert!(options_list.is_ok());
	let options_list = options_list.unwrap();
	
	let message_payload = parse_payload(udp_buffer);
	assert!(message_payload.is_ok());
	let message_payload = message_payload.unwrap();
	
	CoapMessage {
		header:message_header,
		option_list:options_list,
		payload: message_payload
	}
}

fn parse_coap_header(udp_buffer: [u8; MAX_MTU]) -> Result<CoapMessageHeader, &'static str>{
	
	let message_size = find_last_buffer_position(udp_buffer);
	
	if message_size < 4 { //Could not possible have all manditory bytes
		return Err("Invalid header length");
	}
	
	//Get id
	let id_upper_nibble = (udp_buffer[2] as u16) << 8;
	let id_lower_nibble = udp_buffer[3] as u16;
	
	let token_num_of_bytes = 0x0Fu8 & udp_buffer[0] as u8;
	
	if token_num_of_bytes > 8 { //token too big
		return Err("Invalid token length");
	}
	
	
	let coap_token;
	
	match token_num_of_bytes {
		1 => coap_token = udp_buffer[4] as u64,
		2 => coap_token = ((udp_buffer[4] as u64) << 8) | (udp_buffer[5] as u64),
		3 => coap_token = ((udp_buffer[4] as u64) << 16) | ((udp_buffer[5] as u64) << 8) | (udp_buffer[6] as u64),
		4 => coap_token = ((udp_buffer[4] as u64) << 32) | ((udp_buffer[5] as u64) << 16) | ((udp_buffer[6] as u64) << 8) | (udp_buffer[7] as u64),
		_ => coap_token = 0,
	}
	
	let coap_header = CoapMessageHeader {
		version:(0xC0u8 & udp_buffer[0])>>6,
		coap_type:(0x30u8 & udp_buffer[0])>>4,
		token_length:token_num_of_bytes,
		coap_class:(0x0Fu8 & udp_buffer[0]),
		code:(udp_buffer[1]) & 0x1Fu8,
		id:id_upper_nibble|id_lower_nibble,
		token:coap_token
	};
	
	println!("Coap header: {}", coap_header);
	
	Ok(coap_header)
}

fn parse_coap_options(option_start_pos: u8, udp_buffer: [u8; MAX_MTU]) -> Result<LinkedList<Option>, &'static str>{
	
	let option_start = option_start_pos + 4; //Manditory message header is 4 bytes
	let last_pos = find_last_buffer_position(udp_buffer);
	let mut pos = option_start as usize;
	let mut option_list: LinkedList<Option> = LinkedList::new();
	let mut prev_option_number = 0 as u16;
	
	while pos <= last_pos {
		let observed_byte = udp_buffer[pos];
		
		if observed_byte == 0xFF{
			break;
		}
		
		let mut curr_option = Option{
			number:0,
			length:0,
			value:Vec::<u8>::new()
		};
		
		let option_delta = observed_byte >> 4;
		let option_length =  observed_byte & 0x0F;
		
		match option_delta {
			
			13 => { //The option is in the next byte
				curr_option.number = (udp_buffer[pos+1]).into();
				curr_option.number += prev_option_number;
				
				pos += 1; //Update our postion in the buffer
			}
			
			14 => { // The option is in the next 2 bytes
				curr_option.number = ((udp_buffer[pos+1] as u16) << 8) | (udp_buffer[pos+2] as u16);
				curr_option.number += prev_option_number;
				
				pos += 2; //Update our postion in the buffer
			}
			
			15 => { //reserved for late use
				return Err("This option delta is reserved");
			}
			
			_ => {
				curr_option.number = option_delta as u16;
				curr_option.number += prev_option_number; 
			}
		}
		
		prev_option_number = curr_option.number;
		
		match option_length {
			
			13 => { //The length is in the next byte
				curr_option.length = (udp_buffer[pos+1]).into();
				pos += 1; //Update our postion in the buffer
			}
			
			14 => { //The length is in the next two bytes
				curr_option.length = ((udp_buffer[pos+1] as u16) << 8) | (udp_buffer[pos+2] as u16);
				pos += 2; //Update our postion in the buffer
			}
			
			15 => {
				return Err("This option length is reserved");
			}
			
			_ => {
				curr_option.length = option_length.into();
			}
			
		}
		
		curr_option.value = udp_buffer[(pos + 1)..last_pos].to_vec();
		
		option_list.push_back(curr_option);
		
		pos += 1;
	}
	
	Ok(option_list)
	
}

fn parse_payload(udp_buffer: [u8; MAX_MTU]) -> Result<Vec<u8>, &'static str>{
	
	let last_pos = find_last_buffer_position(udp_buffer);
	let mut pos = 0;
	
	while pos <= last_pos{
		if udp_buffer[pos] == 0xFF {
			return Ok(udp_buffer[pos+1..last_pos].to_vec());
		}
		else{
			pos += 1;
		}
	}
	
	Err("Could not find payload")
	
}

fn print_udp_datagram_buffer(udp_buffer: [u8; MAX_MTU]){
	
	
	let mut i = 0;
	while udp_buffer[i] != 0 {
    	println!("{:#x}", udp_buffer[i]);
		i += 1;
	}
}

fn find_last_buffer_position(udp_buffer: [u8; MAX_MTU]) -> usize {
	let mut i = 0 as usize;
	while udp_buffer[i] != 0 {
		i += 1;
	}
	
	i-1
}