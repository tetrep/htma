fn main()
{
  let input = std::io::stdin().read_line()
                .ok()
                .expect("failed to read stdin");

  println!("{}", htma::htparse(input.as_slice()));
}

mod htma
{
  #![warn(experimental)]

  use dma;

  struct Memory
  {
    memory_size: uint,
    memory_address: String,
  }

  enum URIState
  {
    URISpace,
    URIOptionalSlash,
    URISize,
    URIMemory,
  }

  /// `htparse` will take in the first line of an http request and return the specified memory
  /// as a (utf8?) string
  pub fn htparse(input: &str)
  -> String
  {
    let req_mem = tktk_get(input);
    dma::read_memory_address(req_mem.memory_size, req_mem.memory_address.as_slice()).to_string()
  }

  pub fn tktk_get(input: &str)
  -> Memory
  {
    let mut memory = Memory{memory_size: 1, memory_address: "".to_string()};
    let mut temp_memory_size = "".to_string();

    //dat state machine
    let mut state = URISpace;
    for c in input.chars()
    {
      match state
      {
        //find the first space (seperates verb and uri)
        URISpace => {if(' ' == c){state = URIOptionalSlash;}},
        //consume slash if uri starts with it, otherwise treat it as the first character of the size
        URIOptionalSlash => {if('/' != c){memory.memory_address.push(c);}state = URIMemory},
        //get the address of memory we will be using, stopping when we hit a space
        URIMemory => {if('/' != c){memory.memory_address.push(c);}else{state = URISize;}},
        //get the amount of memory will we be using
        URISize => {if(' ' != c){temp_memory_size.push(c);}else{break;}},
      }
    }
    memory
  }
}

mod dma
{
  #![warn(experimental)]

  use std::num;

  pub fn read_memory_address(memory_size: uint, encoded_memory_address: &str)
  -> String
  {
    let decoded_memory_address = hex_str_to_uint(encoded_memory_address);

    (format!("{}", decoded_memory_address))
  }

  // because nothing stable can do le hex >.<
  pub fn hex_str_to_uint(hex_str: &str)
  -> uint
  {
    let mut ret_uint = 0;
    let mut sig = hex_str.len()-1;

    for c in hex_str.chars()
    {
      match c
      {
        '0' => {sig -= 1},
        '1' => {ret_uint = 1*num::pow(16,sig) + ret_uint;},
        '2' => {ret_uint = 2*num::pow(16,sig) + ret_uint;},
        '3' => {ret_uint = 3*num::pow(16,sig) + ret_uint;},
        '4' => {ret_uint = 4*num::pow(16,sig) + ret_uint;},
        '5' => {ret_uint = 5*num::pow(16,sig) + ret_uint;},
        '6' => {ret_uint = 6*num::pow(16,sig) + ret_uint;},
        '7' => {ret_uint = 7*num::pow(16,sig) + ret_uint;},
        '8' => {ret_uint = 8*num::pow(16,sig) + ret_uint;},
        '9' => {ret_uint = 9*num::pow(16,sig) + ret_uint;},
        'a' => {ret_uint = 10*num::pow(16,sig) + ret_uint;},
        'b' => {ret_uint = 11*num::pow(16,sig) + ret_uint;},
        'c' => {ret_uint = 12*num::pow(16,sig) + ret_uint;},
        'd' => {ret_uint = 13*num::pow(16,sig) + ret_uint;},
        'e' => {ret_uint = 14*num::pow(16,sig) + ret_uint;},
        'f' => {ret_uint = 15*num::pow(16,sig) + ret_uint;},
        _   => {ret_uint = 0; break;}, // srsly...
      }

      println!("{}^{} => {}", c, sig, ret_uint);

      sig -= 1;
    }

    ret_uint
  }
}
