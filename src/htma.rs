fn main()
{
  let buffer: &'static str = "Hello, world!";
  println!("{:p}\n{}", &buffer, std::mem::size_of_val(&buffer));

  /*
  let input = std::io::stdin().read_line()
    .ok()
    .expect("failed to read stdin");

  println!("{}", htma::htparse_raw(input.as_slice()));
  */

  println!("listening: 127.0.0.1 1066");
  htmacp::repl("127.0.0.1", "1066", htma::htparse);

  println!("le buffer: {}", buffer);
}

mod htmacp
{
  #![warn(experimental)]

  use std;

  //because traits can't scope....
  use std::io::Listener;
  use std::io::Acceptor;


  fn rep(mut connection: std::io::TcpStream, eval: fn (&str) -> String)
  {
    let mut buffer = [0u8, ..1024];
    connection.read(&mut buffer);

    let buffer_str = match std::str::from_utf8(&mut buffer) {
        Some(s) => s,
        None => "",
      };

    connection.write_str(eval(buffer_str).as_slice());
  }


  pub fn repl(ip: &str, port: &str, eval: fn (&str) -> String)
  {

    let listener = std::io::TcpListener::bind(format!("{}:{}", ip, port).as_slice())
      .ok()
      .expect("failed to bind");
    let mut acceptor = listener.listen()
      .ok()
      .expect("failed to listen");

    for connection in acceptor.incoming()
    {
      match connection
      {
        Err(e) => { println!("weeoweeoweeo: {}", e); }
        Ok(connection) => { rep(connection, eval); }
      }
    }

    drop(acceptor);
  }
}

mod htma
{
  #![warn(experimental)]

  use dma;
  use std;

  struct Memory
  {
    size: uint,
    string: String,
    pointer: *const u8,
  }

  struct Memory_slice
  {
    pointer: *const u8,
    size: uint,
  }

  enum URIState
  {
    URISpace,
    URIOptionalSlash,
    URISize,
    URIMemory,
  }

  /// `htparse` will take in the first line of an http request and return the specified memory
  /// as an http string
  pub fn htparse(input: &str)
  -> String
  {
    let mut req_mem = tktk_get(input);

    req_mem.pointer = dma::get_memory_pointer(req_mem.string.as_slice());

    let http_str = dma::read_memory_pointer(req_mem.pointer, req_mem.size);

    add_headers(http_str.as_slice())
  }

  fn tktk_get(input: &str)
  -> Memory
  {
    let mut memory = Memory { size: 0, string: "".to_string(), pointer: std::ptr::null() };
    let mut size_str = String::new();

    let mut state = URIState::URISpace;
    for c in input.chars()
    {
      match state
      {
        //find the first space (seperates verb and uri)
        URIState::URISpace => { if(' ' == c) { state = URIState::URIOptionalSlash; } },
        //consume slash if uri starts with it, otherwise treat it as the first character of the size
        URIState::URIOptionalSlash => { if('/' != c) { memory.string.push(c); } state = URIState::URIMemory },
        //get the address of memory we will be using, stopping when we hit a space
        URIState::URIMemory => { if('/' != c) { memory.string.push(c); } else { state = URIState::URISize; } },
        //get the amount of memory will we be using
        URIState::URISize => { if(' ' != c) { size_str.push(c); } else { break; } },
      }
    }

    let maybe_num = from_str(size_str.as_slice());
    match maybe_num
    {
      Some(number) => memory.size = number,
      None => memory.size = 0,
    }

    memory
  }

  fn add_headers(body: &str)
  -> String
  {
    //println!("HTTP 200 OK\nContent-Type: text/plain\nContent-Length: {}\n{}", body.len(), body)
    format!("HTTP 200 OK\nContent-Type: text/plain\nContent-Length: {}\n\n{}\n\n", body.len(), body)
  }
}

mod dma
{
  #![warn(experimental)]

  extern crate libc;

  use std;
  use std::num::Int;

  //takes an address and returns the data as a hex encoded string
  pub fn read_memory_pointer(memory_address: *const u8, memory_size: uint)
  -> String
  {
    //mark part of page as read, will fail if we can't actually read the page
    //this way we know if memory address is valid, needs to be page aligned
    let mprotect_result = unsafe
    {
      libc::funcs::posix88::mman::mprotect(((memory_address as u64) - ((memory_address as u64) % 4096)) as *mut libc::c_void,
      ((memory_size as u64) + ((memory_address as u64) % 4096)) as libc::size_t, 0x01 as libc::c_int)
    };

    let mut http_string: String = "".to_string();

    if(-1 == mprotect_result)
    {
      println!("mmap failed; errno = {}", std::os::errno());
      http_string = format!("{:p}", &"Invalid memory address");
    }
    else
    {
      println!("grabbing {} bytes from {}", memory_size, memory_address);

      //http_str = unsafe { std::str::from_c_str(req_mem.pointer) };
      //http_str = unsafe { *(req_mem.pointer as *const &str) };
      for i in std::iter::range(0, memory_size)
      {
        //get the byte
        let byte = unsafe { *(((memory_address as uint) + i) as *const u8) };
        //high byte
        http_string.push(u8_to_hex(byte >> 4));
        //low byte
        http_string.push(u8_to_hex(byte & 0x0f));
      }

      //make it executable! because it might be memory of our process that we need to execute
      unsafe { libc::funcs::posix88::mman::mprotect(((memory_address as u64) - ((memory_address as u64) % 4096)) as *mut libc::c_void,
        ((memory_size as u64) + ((memory_address as u64) % 4096)) as libc::size_t, 0x04 as libc::c_int) };
    }

    http_string
  }

  pub fn get_memory_pointer(encoded_memory_address: &str)
  -> *const u8
  {
    let decoded_memory_address = hex_str_to_uint(encoded_memory_address);
    let p: *const u8 = decoded_memory_address as *const u8;

    p
  }

  // because nothing stable can do hex >.<
  pub fn hex_str_to_uint(hex_str: &str)
  -> uint
  {
    let mut ret_uint = 0;
    let mut sig = hex_str.len();
    let mut trailing_zero = false;

    for c in hex_str.chars()
    {
      ret_uint += ((hex_byte_to_u8(c) as uint) << 4*(sig-1));

      sig -= 1;
    }

    ret_uint
  }

  pub fn hex_byte_to_u8(c: char)
  -> u8
  {
    let mut ret_uint: u8 = 0;
    match c
    {
      '0' => { ret_uint = 0 },
      '1' => { ret_uint = 1 },
      '2' => { ret_uint = 2 },
      '3' => { ret_uint = 3 },
      '4' => { ret_uint = 4 },
      '5' => { ret_uint = 5 },
      '6' => { ret_uint = 6 },
      '7' => { ret_uint = 7 },
      '8' => { ret_uint = 8 },
      '9' => { ret_uint = 9 },
      'a' => { ret_uint = 10 },
      'b' => { ret_uint = 11 },
      'c' => { ret_uint = 12 },
      'd' => { ret_uint = 13 },
      'e' => { ret_uint = 14 },
      'f' => { ret_uint = 15 },
      _   => { ret_uint = 0; }, // we're 0x compatible!
    }

    ret_uint
  }
  pub fn u8_to_hex(byte: u8)
  -> char
  {
    let mut ret_char: char = 'x';
    match byte
    {
      0  => { ret_char = '0' },
      1  => { ret_char = '1' },
      2  => { ret_char = '2' },
      3  => { ret_char = '3' },
      4  => { ret_char = '4' },
      5  => { ret_char = '5' },
      6  => { ret_char = '6' },
      7  => { ret_char = '7' },
      8  => { ret_char = '8' },
      9  => { ret_char = '9' },
      10 => { ret_char = 'a' },
      11 => { ret_char = 'b' },
      12 => { ret_char = 'c' },
      13 => { ret_char = 'd' },
      14 => { ret_char = 'e' },
      15 => { ret_char = 'f' },
      _  => { ret_char = 'x' },
    }

    ret_char
  }
}
