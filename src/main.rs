use std::convert::TryInto;
use std::fs::create_dir_all;
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::collections::LinkedList;
use std::path::Path;
use std::fs::File;
use std::env;

const MAX_MTU: usize = 15000; //MTU assumed to be 1500

const CLASS_METHOD: u8 = 0; 
const CODE_EMPTY: u8 = 0;
const CODE_GET: u8 = 1;
const CODE_POST: u8 = 2;
const CODE_PUT: u8 = 3;
const CODE_DELETE: u8 = 4;

const CLASS_SUCCESS: u8 = 2;
const CODE_CREATED: u8 = 1;
const CODE_DELETED: u8 = 2;

const CLASS_CLIENT_ERROR: u8 = 4;
const CODE_BAD_REQUEST: u8 = 0;
const CODE_NOT_FOUND: u8 = 4;
const CODE_METHOD_NOT_ALLOWED: u8 = 5;
const CODE_CONFLICT: u8 = 9;

const CLASS_SERVER_ERROR: u8 = 5;
const CLASS_SIGNALING_CODES: u8 = 7;

///A structure for saving the CoAP header
#[derive(Clone)]
struct CoapMessageHeader {
	version:u8,
	coap_type:u8,
	token_length:u8,
	coap_class:u8,
	code:u8,
	id:u16,
	token:Vec<u8>
}

///A structure for saving the entire CoAP message
#[derive(Clone)]
struct CoapMessage{
	header:CoapMessageHeader,
	option_list:LinkedList<Option>,
	payload: Vec<u8> 
}

///A structure for the CoAP message
#[derive(Clone)]
struct Option{
	number:u16,
	length:u16,
	value: Vec<u8>
}

impl std::fmt::Display for CoapMessageHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(Header contains, version: {:#x}, type: {:#x} , token length: {:#x}, class: {:#x}, code: {:#x}, id: {:#x}, token: {:?})", self.version, self.coap_type, self.token_length, self.coap_class, self.code, self.id, self.token)
    }
}

impl std::fmt::Display for Option {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(Option contains, number: {}, length: {}, value: {:#X?}", self.number, self.length, self.value)
    }
}

fn main() {
    println!("Hello, world!");

	let result = create_storage_dir();
	assert!(result.is_ok());

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

fn handle_request(socket: UdpSocket, udp_buffer: [u8; MAX_MTU], sender_adress: SocketAddr){
	//Get the coap message
	let message = parse_message_from_datagram(udp_buffer);
	
	//execute message request
	println!("Executing message!");
	let result = execute_request(message, socket, sender_adress);
	assert!(result.is_ok());
}

fn create_storage_dir() -> std::io::Result<()> {
	
	let dir = get_storage_dir_as_string();
	
	create_dir_all(dir)?;
	
	Ok(())
}

fn execute_request(message:CoapMessage, socket: UdpSocket, sender_adress: SocketAddr) -> Result<&'static str, &'static str>{
	
	match message.header.coap_class {
		
		CLASS_METHOD => { //method
			match message.header.code {
				CODE_EMPTY => { //Empty
					println!("Got method code: {}", message.header.code);
					send_coap_response(CLASS_CLIENT_ERROR, CODE_METHOD_NOT_ALLOWED, message, socket, sender_adress);
				}
				CODE_GET => { //GET
					println!("Got method code: {}", message.header.code);
					send_coap_response(CLASS_CLIENT_ERROR, CODE_METHOD_NOT_ALLOWED, message, socket, sender_adress);
				}
				CODE_POST => { //POST
					println!("Got method code: {}", message.header.code);
					handle_post_request(message, socket, sender_adress)?;
				}
				CODE_PUT => { //PUT
					println!("Got method code: {}", message.header.code);
					send_coap_response(CLASS_CLIENT_ERROR, CODE_METHOD_NOT_ALLOWED, message, socket, sender_adress);
				}
				CODE_DELETE => { //DELETE
					println!("Got method code: {}", message.header.code);
					send_coap_response(CLASS_CLIENT_ERROR, CODE_METHOD_NOT_ALLOWED, message, socket, sender_adress);
				}
				_ => {
					println!("Got method code: {}", message.header.code);
					send_coap_response(CLASS_CLIENT_ERROR, CODE_METHOD_NOT_ALLOWED, message, socket, sender_adress);
				}
			}
		}
		
		CLASS_SUCCESS => { //success
			println!("Got success code: {}", message.header.code);
			match message.header.code {
				_ => {
					println!("Got success code: {}", message.header.code);
					send_coap_response(CLASS_CLIENT_ERROR, CODE_BAD_REQUEST, message, socket, sender_adress);
				}
			}
		}
		
		CLASS_CLIENT_ERROR => {
			println!("Got client error code: {}", message.header.code);
			send_coap_response(CLASS_CLIENT_ERROR, CODE_BAD_REQUEST, message, socket, sender_adress);
		}
		
		CLASS_SERVER_ERROR => {
			println!("Got server error code: {}", message.header.code);
			send_coap_response(CLASS_CLIENT_ERROR, CODE_BAD_REQUEST, message, socket, sender_adress);
		}
		
		CLASS_SIGNALING_CODES => {
			println!("Got signal code: {}", message.header.code);
			send_coap_response(CLASS_CLIENT_ERROR, CODE_BAD_REQUEST, message, socket, sender_adress);
		}
		
		_ => {
			println!("Got strange code: {}", message.header.code);
			send_coap_response(CLASS_CLIENT_ERROR, CODE_BAD_REQUEST, message, socket, sender_adress);
			return Err("Got code that is not defined!");
		}
	}
	
	Ok("OK")
}

fn handle_post_request(message:CoapMessage, socket: UdpSocket, sender_adress: SocketAddr) -> Result<&'static str, &'static str>{
	assert_eq!(message.header.coap_class, 0); //make sure the message is a method
	assert_eq!(message.header.code, 2); //make sure the message is a POST
	
	let path_string = parse_options_to_path(message.clone().option_list)?;
	let code;
	let class;
	
	println!("The file is: {}", path_string);
	
	if Path::new(&path_string).exists(){
		//File already exists		
		class = CLASS_CLIENT_ERROR;
		code = CODE_CONFLICT;
	}
	else {
		let file = File::create(&path_string);
		assert!(file.is_ok());
		class = CLASS_SUCCESS;
		code = CODE_CREATED;
	}
	
	send_coap_response(class, code, message, socket, sender_adress);
	Ok("OK")
}

fn send_coap_response(class: u8, code: u8, message:CoapMessage, socket: UdpSocket, sender_adress: SocketAddr){
	
	let buffer = create_response_as_buffer(class, code, message);
	
	let send_response = socket.send_to(&buffer, &sender_adress);
	assert!(send_response.is_ok());
	
}

fn create_response_as_buffer(class: u8, code: u8, mut message:CoapMessage) -> Vec<u8>{
	
	let mut buffer: Vec<u8> = Vec::new();
	
	buffer.push(message.header.version << 6 | 0x02 << 4 | message.header.token_length); // version, type; token length
	
	buffer.push(class << 5 | code); //Class and code
	
	buffer.push(((message.header.id & 0xF0) >> 8).try_into().unwrap());

	buffer.push((message.header.id & 0x0F).try_into().unwrap());
	
	buffer.append(&mut message.header.token); //token ID.
	
	buffer
}

fn get_storage_dir_as_string() -> String{
	let path = env::current_dir();
	assert!(path.is_ok());
	let path = path.unwrap();
	let path = path.into_os_string().into_string();
	assert!(path.is_ok());
	let mut path = path.unwrap();
	
	path.push_str("/Storage");
	
	path
	
}

fn parse_options_to_path(option_list:LinkedList<Option>) -> Result<String, &'static str>{
	
	
	
	let mut file_location = get_storage_dir_as_string();
	let mut list = option_list;
	
	println!("Starting with: {}. List contains {}", file_location, list.len());
	
	while !list.is_empty() {
		
		let curr_option = list.pop_front().unwrap();
		
		println!("Examining option: {}", curr_option);
		
		match curr_option.number {
			
			3 => { // the Uri-Host Option specifies the Internet host of the resource being requested
				let host = String::from_utf8(curr_option.value);
				assert!(host.is_ok());	
				let host = host.unwrap();		
								
				if !host.contains("localhost") & !host.contains("127.0.0.1") {
					println!("Wrong server: {}", host);
					return Err("Wrong server!");
				}
			}
			
			11 => { // each Uri-Path Option specifies one segment of the absolute path to the resource
				let location_part = String::from_utf8(curr_option.value);
				assert!(location_part.is_ok());	
				let location_part = location_part.unwrap();
				
				file_location.push_str("/");
				
				println!("The appending: {}, Total path: {}",location_part, file_location);
				
				file_location.push_str(&location_part);
			}
			
			_ => {
				println!{"Not implemented yet! code: {}", curr_option.number}
				return Err("Not implemented yet!");
			}
		}
		
		println!("Options left: {}", list.len());
	}
	
	if file_location == "Storage"{
		eprintln!("No path found!");
		Err("No path found!")
	}
	else{
		Ok(file_location)
	}
	
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
	
	CoapMessage {
		header:message_header,
		option_list:options_list,
		payload: message_payload
	}
}


/// Returns the header of the CoAP message in the buffer.
///
/// # Arguments
///
/// * `udp_buffer` - The buffer to be printed.
/// 
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
	
	
	let coap_token = udp_buffer[4..4+(token_num_of_bytes as usize)].to_vec();
	
	let coap_header = CoapMessageHeader {
		version:(0xC0u8 & udp_buffer[0])>>6,
		coap_type:(0x30u8 & udp_buffer[0])>>4,
		token_length:token_num_of_bytes,
		coap_class:(0x07u8 & (udp_buffer[1]>>5)),
		code:(udp_buffer[1]) & 0x1Fu8,
		id:id_upper_nibble|id_lower_nibble,
		token:coap_token
	};
	
	println!("Coap header: {}", coap_header);
	
	Ok(coap_header)
}

/// Returns the payload of the CoAP message or an empty vector if no payload is found.
///
/// # Arguments
///
/// * `option_start_pos` - The position where the option starts in the buffer.
/// * `udp_buffer` - The buffer containing a correct CoAP message.
/// 
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
				eprintln!("This option delta is reserved");
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
				eprintln!("This option length is reserved");
				return Err("This option length is reserved");
			}
			
			_ => {
				curr_option.length = option_length.into();
				pos += 1;
			}
			
		}
		
		if option_length > 0{
			curr_option.value = udp_buffer[pos..(pos+(option_length as usize))].to_vec();
		
			println!("Found option: {}", curr_option);
			
			option_list.push_back(curr_option);
			
			pos += option_length as usize;
		}
		
	}
	
	Ok(option_list)
	
}


/// Returns the payload of the CoAP message or an empty vector if no payload is found.
///
/// # Arguments
///
/// * `udp_buffer` - The buffer containing a correct CoAP message.
/// 
fn parse_payload(udp_buffer: [u8; MAX_MTU]) -> Vec<u8>{
	
	let last_pos = find_last_buffer_position(udp_buffer);
	let mut pos = 0;
	
	while pos <= last_pos{
		if udp_buffer[pos] == 0xFF {
			return udp_buffer[pos+1..last_pos].to_vec();
		}
		else{
			pos += 1;
		}
	}
	
	Vec::new()
	
}

/// Print the buffer to the standard output.
///
/// # Arguments
///
/// * `udp_buffer` - The buffer to be printed.
/// 
fn print_udp_datagram_buffer(udp_buffer: [u8; MAX_MTU]){
	
	
	let mut i = 0;
	while udp_buffer[i] != 0 {
    	println!("{:#x}", udp_buffer[i]);
		i += 1;
	}
}

/// Returns the last position of the buffer.
///
/// # Arguments
///
/// * `udp_buffer` - The buffer to be printed.
/// 
fn find_last_buffer_position(udp_buffer: [u8; MAX_MTU]) -> usize {
	let mut i = 0 as usize;
	while udp_buffer[i] != 0 {
		i += 1;
	}
	
	i-1
}