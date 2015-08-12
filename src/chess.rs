use rustc_serialize::json;
use std::vec::Vec;
use std::collections::HashMap;
use std::fs::{
	File,
	OpenOptions,
};
use std::io;
use std::io::{
	Write,
	Read,
};

#[derive(RustcDecodable,RustcEncodable)]
struct ChessMap {
    pub moves: 				Vec<String>,
    board:				HashMap<u64, String>,
    pub discarded_pieces:	Vec<String>,
    pub player_white:		String,
    pub player_black:		String,
}

/**
*The coordinate system is a standard chess board but in the octal numbering system
*The origin in the lower left
*	0oXY
*	X is the letter starting with a=0 and h=7
*	Y is the number-1, that is Vertical_Coordinate=Y+1
*	So for example A4 would be 0o03 and E1 would be 0o40
***/
impl ChessMap {
	pub fn new(white_name: String, black_name: String) -> ChessMap {
		let mut new_board = HashMap::new();
		for letter in 0..8{
			new_board.insert(0o01+(letter*0o10), "wpawn".to_string());
			new_board.insert(0o06+(letter*0o10), "bpawn".to_string());
		}
		new_board.insert(0o00, "wrook".to_string());
		new_board.insert(0o10, "wknight".to_string());
		new_board.insert(0o20, "wbishop".to_string());
		new_board.insert(0o30, "wqueen".to_string());
		new_board.insert(0o40, "wking".to_string());
		new_board.insert(0o50, "wbishop".to_string());
		new_board.insert(0o60, "wknight".to_string());
		new_board.insert(0o70, "wrook".to_string());
		new_board.insert(0o07, "brook".to_string());
		new_board.insert(0o17, "bknight".to_string());
		new_board.insert(0o27, "bbishop".to_string());
		new_board.insert(0o37, "bqueen".to_string());
		new_board.insert(0o47, "bking".to_string());
		new_board.insert(0o57, "bbishop".to_string());
		new_board.insert(0o67, "bknight".to_string());
		new_board.insert(0o77, "brook".to_string());
		ChessMap{
			player_black: 		black_name,
			player_white: 		white_name,
			discarded_pieces:	Vec::new(),
			moves: 				Vec::new(),
			board:				new_board,
		}
	}

	pub fn legal_moves(&self, location: u64) -> Vec<u64> {
		let mut valid_moves = Vec::new();
        if location >= 0o100{
            return valid_moves;
        }
		let piece = match (&self).board.get(&location) {
			Some(x) 	=> 	x,
			None		=>	return valid_moves,
		};
		let alignment = &piece[0..1];
		let piece_type = &piece[1..piece.len()];
		let (horizontal_place, vertical_place) = ChessMap::unpack_horizontal_vertical(location);
		if piece_type == "rook"{
			valid_moves = (&self).search_cross(alignment,location);
			//
		}else if piece_type == "knight"{
			match (&self).check_relative_space(alignment,location,-2, 1){
				Some(x)	=>	valid_moves.push(x),
				None	=>	(),
			};
			match (&self).check_relative_space(alignment,location,-2,-1){
				Some(x)	=>	valid_moves.push(x),
				None	=>	(),
			};
			match (&self).check_relative_space(alignment,location,-1, 2){
				Some(x)	=>	valid_moves.push(x),
				None	=>	(),
			};
			match (&self).check_relative_space(alignment,location,-1,-2){
				Some(x)	=>	valid_moves.push(x),
				None	=>	(),
			};
			match (&self).check_relative_space(alignment,location, 1, 2){
				Some(x)	=>	valid_moves.push(x),
				None	=>	(),
			};
			match (&self).check_relative_space(alignment,location, 1,-2){
				Some(x)	=>	valid_moves.push(x),
				None	=>	(),
			};
			match (&self).check_relative_space(alignment,location, 2, 1){
				Some(x)	=>	valid_moves.push(x),
				None	=>	(),
			};
			match (&self).check_relative_space(alignment,location, 2,-1){
				Some(x)	=>	valid_moves.push(x),
				None	=>	(),
			};
		}else if piece_type == "bishop"{
			valid_moves = (&self).search_x(alignment, location);
			//
		}else if piece_type == "pawn"{
			let vertical_offset = if alignment == "w" && vertical_place+1 < 0o10{
				1
			} else{
				-1
			};
			let forward_location = (horizontal_place*0o10)+(vertical_place+vertical_offset);
			if (&self).space_valid(alignment, forward_location){
				valid_moves.push(forward_location)
			}
			if horizontal_place+1 < 0o10{
				let forward_right = forward_location+0o10;
				match (&self).board.get(&forward_right) {
					Some(_) => {
						if (&self).space_valid(alignment, forward_right){
							valid_moves.push(forward_right);
						}
					},
					None	=> (),
				};
			}else if horizontal_place >= 1{
				let forward_left = forward_location-0o10;
				match (&self).board.get(&forward_left) {
					Some(_) 	=> {
						if (&self).space_valid(alignment, forward_left){
							valid_moves.push(forward_left);
						}
					},
					None	=> (),
				};
			}
		}else if piece_type == "queen"{
			valid_moves.extend((&self).search_cross(alignment, location));
			valid_moves.extend((&self).search_x(alignment, location));
		}else if piece_type == "king"{
			let mut potential_moves = (&self).check_immediate_moves(alignment, location);
			if potential_moves.is_empty(){
				return valid_moves;
			}
			for (location_scan, piece) in (&self).board.iter(){
				let alignment_scan = &piece[0..1];
				if alignment_scan == alignment{
					continue;
				}
				if &piece[1..piece.len()] == "king"{
					let potential_moves_scan = (&self).check_immediate_moves(alignment_scan, *location_scan);
					for x in potential_moves_scan{
						if potential_moves.contains(&x){
							let index = potential_moves.iter().position(|&i| i==x).unwrap();
							potential_moves.remove(index);
						}
					}
				}else{
					let potential_moves_scan = (&self).legal_moves(*location_scan);
					for x in potential_moves_scan{
						if potential_moves.contains(&x){
							let index = potential_moves.iter().position(|&i| i==x).unwrap();
							potential_moves.remove(index);
						}
					}
				}
			}
			valid_moves.extend(potential_moves);
		}
		valid_moves
	}

	pub fn move_piece(&mut self, current_location: u64, new_location: u64) -> bool {
		let allowed_moves = (&self).legal_moves(current_location);
		if allowed_moves.contains(&new_location) == false{
			return false;
		}
		let piece = match self.board.remove(&current_location) {
			Some(x) => x,
			None	=> return false,
		};
		match self.board.insert(new_location, piece){
			Some(x)	=>	{
				self.kill_piece(new_location);
				match self.board.insert(new_location, x){
					Some(_)	=> 	panic!("Attempted to move piece from 0o{:o} to 0o{:o}", current_location, new_location),
					None	=>	return true,
				}
			},
			None	=>	return true,
		};
	}

	pub fn save(chess_board: ChessMap) -> io::Result<()>{
		let data = json::encode(&chess_board).unwrap();
		let mut file = match OpenOptions::new()
						.write(true)
						.truncate(true)
						.open(format!("{}_{}", chess_board.player_white, chess_board.player_black)){
							Ok(x)	=>	x,
							Err(x)	=>	return Err(x),
						};
		file.write_all(data.as_bytes())
	}

	pub fn from_file(player_white: String, player_black: String) -> Option<ChessMap> {
		let mut file = match OpenOptions::new()
					.read(true)
					.open(format!("{}_{}", player_white, player_black)){
						Ok(x)	=>	x,
						Err(_)	=>	return None,
					};
		let mut data = String::new();
		match file.read_to_string(&mut data){
			Ok(x)	=>	{
				if x < 3{
					return None;
				}
			},
			Err(_)	=>	return None,
		};
		let ret: ChessMap = match json::decode(&data[0..data.len()]){
			Ok(x)	=>	x,
			Err(x)	=>	return None,
		};
		Some(ret)
	}

	fn search_cross(&self, alignment: &str, location: u64) -> Vec<u64> {
		let mut valid_moves = Vec::new();
		//search to the right
		valid_moves.extend((&self).search_ray(alignment, location, 1, 0));
		//search to the left
		valid_moves.extend((&self).search_ray(alignment, location,-1, 0));
		//search above
		valid_moves.extend((&self).search_ray(alignment, location, 0, 1));
		//search below
		valid_moves.extend((&self).search_ray(alignment, location, 0,-1));
		valid_moves
	}

	fn search_x(&self, alignment: &str, location: u64) -> Vec<u64> {
		let mut valid_moves = Vec::new();
		//search upper right
		valid_moves.extend((&self).search_ray(alignment, location, 1, 1));
		//search upper left
		valid_moves.extend((&self).search_ray(alignment, location,-1, 1));
		//search lower right
		valid_moves.extend((&self).search_ray(alignment, location, 1,-1));
		//search lower left
		valid_moves.extend((&self).search_ray(alignment, location,-1,-1));
		valid_moves
	}

	fn space_valid(&self, alignment: &str, location: u64) -> bool {
		match (&self).board.get(&location) {
			Some(x) => {
				if &x[0..1] == alignment{false}
				else if &x[1..x.len()] == "king"{false}
				else{true}
			},
			None	=>	true
		}
	}

	fn kill_piece(&mut self, location: u64) -> bool {
		let piece = match self.board.remove(&location) {
			Some(x) => x,
			None	=> return false,
		};
		self.discarded_pieces.push(piece);
		true
	}

	fn unpack_horizontal_vertical(location: u64) -> (u64,u64) {
		let horizontal_place = location/0o10;
		let vertical_place = location%0o10;
		(horizontal_place, vertical_place)
	}

	fn unpack_horizontal_vertical_signed(location: u64) -> (i64,i64) {
		let (horizontal_place, vertical_place) = ChessMap::unpack_horizontal_vertical(location);
		(horizontal_place as i64, vertical_place as i64)
	}

	fn search_ray(&self,alignment: &str, location: u64, step_x: i64, step_y: i64) -> Vec<u64> {
		let mut valid_moves: Vec<u64> = Vec::new();
		for step in 1..8 {
			let (horizontal_place, vertical_place) = ChessMap::unpack_horizontal_vertical_signed(location);
			if horizontal_place+(step_x*step) < 0 || horizontal_place+(step_x*step) >= 0o10{
				return valid_moves
			}
			if vertical_place+(step_y*step) < 0|| vertical_place+(step_y*step) >= 0o10{
				return valid_moves
			}
			let horizontal_scan = horizontal_place+(step_x*step);
			let vertical_scan = vertical_place+(step_y*step);
			let location_scan = (horizontal_scan*0o10)+vertical_scan;
			if (&self).space_valid(alignment, location_scan as u64){
				valid_moves.push(location_scan as u64);
			}
			match (&self).board.get(&(location_scan as u64)){
				Some(_)	=> 	return valid_moves,
				None	=>	continue,
			}
		}
		valid_moves
	}

	fn check_relative_space(&self,alignment: &str, location: u64, x_offset: i64, y_offset: i64) -> Option<u64> {
		let (horizontal_place, vertical_place) = ChessMap::unpack_horizontal_vertical_signed(location);
		if horizontal_place+x_offset < 0 || horizontal_place+x_offset >=0o10{
			return None;
		}else if vertical_place+y_offset < 0 || vertical_place+y_offset >=0o10{
			return None;
		}
		let location_scan = (((horizontal_place+x_offset)*0o10)+(vertical_place+y_offset)) as u64;
		if (&self).space_valid(alignment, location_scan){
			Some(location_scan)
		}else{
			return None;
		}
	}

	fn check_immediate_moves(&self, alignment: &str, location: u64) -> Vec<u64> {
		let mut potential_moves: Vec<u64> = Vec::new();
		for step in -1..2{
			match (&self).check_relative_space(alignment, location, step,-1) {
				Some(x) => 	potential_moves.push(x),
				None	=>	(),
			}
			match (&self).check_relative_space(alignment, location, step, 0) {
				Some(x) => 	potential_moves.push(x),
				None	=>	(),
			}
			match (&self).check_relative_space(alignment, location, step, 1) {
				Some(x) => 	potential_moves.push(x),
				None	=>	(),
			}
		}
		potential_moves
	}

}

#[test]
fn legal_rook_test() {
	let mut test_board = ChessMap::new("blah".to_string(), "didles".to_string());
	if test_board.kill_piece(0o76) == false{
		panic!("Not able to kill pawn in front of rook");
	}
	let rook_moves = test_board.legal_moves(0o77);
	assert_eq!(rook_moves.len(), 6);
	assert!(rook_moves.contains(&0o76));
	assert!(rook_moves.contains(&0o75));
	assert!(rook_moves.contains(&0o74));
	assert!(rook_moves.contains(&0o73));
	assert!(rook_moves.contains(&0o72));
	assert!(rook_moves.contains(&0o71));
}

#[test]
fn legal_bishop_test(){
	let mut test_board = ChessMap::new("blah".to_string(), "asdfa".to_string());
	if test_board.kill_piece(0o16) == false{
		panic!("Not able to kill pawn next to bishop");
	}
	let bishop_moves = test_board.legal_moves(0o27);
	assert_eq!(bishop_moves.len(), 2);
	assert!(bishop_moves.contains(&0o05));
	assert!(bishop_moves.contains(&0o16));
}

#[test]
fn legal_knight_test(){
	let test_board = ChessMap::new("blah".to_string(), "daf".to_string());
	let knight_moves = test_board.legal_moves(0o60);
	assert_eq!(knight_moves.len(), 2);
	assert!(knight_moves.contains(&0o52));
	assert!(knight_moves.contains(&0o72));
}

#[test]
fn legal_pawn_test(){
	let test_board = ChessMap::new("blah".to_string(), "asd".to_string());
	let pawn_moves = test_board.legal_moves(0o11);
	assert_eq!(pawn_moves.len(), 1);
	assert!(pawn_moves.contains(&0o12));
}

#[test]
fn legal_king_test(){
	let mut test_board = ChessMap::new("ahlsdf".to_string(), "asdf".to_string());
	let king_cant_move = test_board.legal_moves(0o47);
	assert!(king_cant_move.is_empty());
	assert!(test_board.kill_piece(0o46));
	let king_can_move = test_board.legal_moves(0o47);
	assert_eq!(king_can_move.len(), 1);
	assert_eq!(*(king_can_move.get(0).unwrap()),0o46);
	assert!(test_board.kill_piece(0o31));
	assert!(test_board.move_piece(0o30,0o35));
	let king_can_no_longer_move = test_board.legal_moves(0o47);
	assert!(king_can_no_longer_move.is_empty());
}

#[test]
fn kill_piece_test(){
	let mut test_board = ChessMap::new("blah".to_string(), "asdf".to_string());
	assert_eq!(test_board.kill_piece(0o76),true);
	assert_eq!(test_board.discarded_pieces.len(),1);
	assert!(test_board.discarded_pieces.contains(&("bpawn".to_string())));
}

#[test]
fn move_piece_test(){
	let mut test_board = ChessMap::new("absldf".to_string(), "asldf".to_string());
	assert!(test_board.kill_piece(0o71));
	assert!(test_board.move_piece(0o70,0o75));
	assert!(test_board.move_piece(0o75,0o76));
	assert_eq!(test_board.discarded_pieces.len(), 2);
}

#[test]
fn save_load_test(){
	let player_white = "test1".to_string();
	let player_black = "test2".to_string();
	let mut test_board = ChessMap::new(player_white.clone(), player_black.clone());
	assert!(test_board.kill_piece(0o77));
	assert_eq!(test_board.discarded_pieces.len(), 1);
	ChessMap::save(test_board).unwrap();
	let mut load_board = ChessMap::from_file(player_white.clone(), player_black.clone()).unwrap();
	assert_eq!(load_board.discarded_pieces.len(), 1);
	assert_eq!(*(load_board.discarded_pieces.get(0).unwrap()), "brook");
}
