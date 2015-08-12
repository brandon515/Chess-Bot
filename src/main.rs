use std::io::prelude::*;
use std::net::{
	TcpStream,
	SocketAddrV4,
	Ipv4Addr,
};
use std::io;
use std::thread;
use std::vec::Vec;

fn get_nick() -> &'static str {
	"BotManJohnson"
}

fn get_channel() -> &'static str {
	"#bottester"
}

fn parse_command(cmd: &String) -> Option<String> {
	let cmd_begin = match cmd.find(|x| x == '!'){
		Some(x)	=>	x,
		None	=>	return None,
	};
	let cmd_end = match cmd.find(|x| x == ' ' || x == '\r'){
		Some(x)	=>	x,
		None	=>	return None,
	};
	let cmd_type = &cmd[cmd_begin..cmd_end];
	if cmd_type == "!version"{
		Some(format!("{} V. 0.3.0", get_nick()))
	}else if cmd_type == "!new"{
        None
    }
    else{
		None
	}
}

fn parse_message(msg: String) -> Option<String>{
	let mut msg_parts: Vec<String> = Vec::new();
	for part in msg.split(" "){
		msg_parts.push(part.to_string());
	};
	if msg.find("PING") != None{
        let res_begin = match msg.find(" "){
            Some(x) =>  x,
            None    =>  return None,
        };
        let res_end = match msg.find("\r\n"){
            Some(x) =>  x,
            None    =>  return None,
        };
		let response_message = format!("PONG {}", &msg[res_begin..res_end]);
		return Some(response_message);
	}else if msg_parts.get(1).unwrap() == "PRIVMSG"{
		let target = msg_parts.get(2).unwrap();
		//dont respond to PMs only public messages
		if target == get_channel(){
			let cmd = msg_parts.get(3).unwrap();
			match parse_command(cmd){
				Some(response_body)	=>	{
					let response_message = format!("PRIVMSG {} :{}\r\n", get_channel(), response_body);
					return Some(response_message);
				}
				None				=>	return None,
			};
		}
	}
	None
}

fn convert_to_server(input: &str) -> String {
	input.to_string()
}


fn main() {
	//Test at weber.freenode.net which is the freenode server in san jose
	let ip_address = SocketAddrV4::new(Ipv4Addr::new(162,213,39,42), 6666);
	let mut socket_stream = TcpStream::connect(ip_address).unwrap();
	let mut thread_stream = socket_stream.try_clone().unwrap();
	thread::spawn(move || {
		let mut buffer_leftover = String::new();
		loop{
			let mut buffer: [u8; 512] = [0; 512];
			let bytes = thread_stream.read(&mut buffer).unwrap();
			//the [0..bytes-2] is to cut off the trailing zeroes and the \r\n that fucks with the formatting
			let buffer_string = if bytes != 0 && buffer[bytes-1..bytes] == [13,10]{
				String::from_utf8_lossy(&buffer[0..bytes-2])
			}else{
				String::from_utf8_lossy(&buffer[0..bytes])
			};
			//cuts off any \r\n that are sent in the same message
			for line in buffer_string.split("\n"){
				//if it contains a \r that means the split split it at a \r\n which is the end of an irc message
				if line.ends_with("\r"){
					let msg = buffer_leftover.clone() + line;
					//clear out the buffer overflow so we dont repeat the same message 30,000 times
					buffer_leftover = "".to_string();
					match parse_message(msg.clone()) {
						Some(x) => 	{
							println!("{}", msg);
							print!("{}", x);
							let _ = thread_stream.write(x.as_bytes()).unwrap();
						}
						None	=>	{
							println!("{}", msg);
							let _ = io::stdout().flush();
						}
					};
				}else{
					//if it doesn't end in this packet then tack it on to the beginning of the next wave
					buffer_leftover = line.to_string();
				}
			}
		}
	});
	let _ = socket_stream.write(format!("NICK {}\r\nUSER {} 0 * :Brandon\r\n", get_nick(), get_nick()).as_bytes()).unwrap();
	let _ = socket_stream.write(format!("JOIN :{}\r\n", get_channel()).as_bytes()).unwrap();
	loop {
		let terminal_input: &mut String = &mut String::new();
		let _ = io::stdin().read_line(terminal_input);
		println!("Terminal input: {}", terminal_input);
		let command = terminal_input.trim();
		if command == "exit"{
			break;
		}
		let server_message = convert_to_server(command);
	}
}
