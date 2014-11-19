fn main()
{
  let buffer = "Hello, world!";
  println!("{:p}\n{}", (&buffer), buffer.len());

  let input = std::io::stdin().read_line()
                .ok()
                .expect("failed to read stdin");

  //println!("{}", htma::htparse(input.as_slice()));
  //println!("le buffer: {}", buffer);
  println!("check direct reference read");
  let mut s = String::new();
  for c in std::iter::range(0, buffer.len())
  {
    //let p: *mut u8 = unsafe { std::mem::transmute(&buffer) };
    //s.push(unsafe { std::ptr::read(p) } as char );
  }

  println!("{}", s);
  println!("still there?\n{}", buffer);
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
  /// as a utf8 string
  pub fn htparse(input: &str)
  -> String
  {
    let req_mem = tktk_get(input);
    dma::read_memory_address(req_mem.memory_size, req_mem.memory_address.as_slice()).to_string()
  }

  pub fn tktk_get(input: &str)
  -> Memory
  {
    let mut memory = Memory { memory_size: 0, memory_address: "".to_string() };
    let mut memory_size_str = String::new();

    //dat state machine
    let mut state = URISpace;
    for c in input.chars()
    {
      match state
      {
        //find the first space (seperates verb and uri)
        URISpace => { if(' ' == c) { state = URIOptionalSlash; } },
        //consume slash if uri starts with it, otherwise treat it as the first character of the size
        URIOptionalSlash => { if('/' != c) { memory.memory_address.push(c); } state = URIMemory },
        //get the address of memory we will be using, stopping when we hit a space
        URIMemory => { if('/' != c) { memory.memory_address.push(c); } else { state = URISize; } },
        //get the amount of memory will we be using
        URISize => { if(' ' != c) { memory_size_str.push(c); } else { break; } },
      }
    }

    let maybe_num = from_str(memory_size_str.as_slice());
    match maybe_num
    {
      Some(number) => memory.memory_size = number,
      None => memory.memory_size = 0,
    }

    memory
  }
}

mod dma
{
  #![warn(experimental)]

  use std;

  pub fn read_memory_address(memory_size: uint, encoded_memory_address: &str)
  -> String
  {
    let decoded_memory_address = hex_str_to_uint(encoded_memory_address);
    let p = std::raw::Slice {
        data: (decoded_memory_address as *const u8),
        len: memory_size
        };
    //let ret: &[u8] = unsafe { std::mem::transmute(p) };
    let mut ret = String::new();

    //format!("ptr: {}\nu8: {}", unsafe { std::ptr::read(decoded_memory_address as *const u8) }, ret)
    for i in std::iter::range(0, memory_size)
    {
      ret.push(unsafe { std::ptr::read((decoded_memory_address + i) as &&str) } as char);
    }

    ret
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
        '1' => {ret_uint = 1*std::num::pow(16,sig) + ret_uint;},
        '2' => {ret_uint = 2*std::num::pow(16,sig) + ret_uint;},
        '3' => {ret_uint = 3*std::num::pow(16,sig) + ret_uint;},
        '4' => {ret_uint = 4*std::num::pow(16,sig) + ret_uint;},
        '5' => {ret_uint = 5*std::num::pow(16,sig) + ret_uint;},
        '6' => {ret_uint = 6*std::num::pow(16,sig) + ret_uint;},
        '7' => {ret_uint = 7*std::num::pow(16,sig) + ret_uint;},
        '8' => {ret_uint = 8*std::num::pow(16,sig) + ret_uint;},
        '9' => {ret_uint = 9*std::num::pow(16,sig) + ret_uint;},
        'a' => {ret_uint = 10*std::num::pow(16,sig) + ret_uint;},
        'b' => {ret_uint = 11*std::num::pow(16,sig) + ret_uint;},
        'c' => {ret_uint = 12*std::num::pow(16,sig) + ret_uint;},
        'd' => {ret_uint = 13*std::num::pow(16,sig) + ret_uint;},
        'e' => {ret_uint = 14*std::num::pow(16,sig) + ret_uint;},
        'f' => {ret_uint = 15*std::num::pow(16,sig) + ret_uint;},
        _   => {ret_uint = 0; break;}, // srsly...
      }

      println!("{}^{} => {}", c, sig, ret_uint);

      sig -= 1;
    }

    ret_uint
  }
}
