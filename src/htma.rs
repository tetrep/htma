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

  enum URI_State
  {
    URI_Space,
    URI_Optional_Slash,
    URI_Size,
    URI_Memory,
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
    let mut state = URI_Space;
    for c in input.chars()
    {
      match state
      {
        //find the first space (seperates verb and uri)
        URI_Space => if(' ' == c){state = URI_Optional_Slash;},
        //consume slash if uri starts with it, otherwise treat it as the first character of the size
        URI_Optional_Slash => if('/' != c){temp_memory_size.push(c);}state = URI_Size;,
        //get the amount of memory will we be using
        URI_Size => if('/' != c){temp_memory_size.push(c);}else{state = URI_Memory;},
        //get the address of memory we will be using, stopping when we hit a space
        URI_Memory => if(' ' != c){memory.memory_address.push(c);}else{break;},
      }
    }
    memory
  }
}

mod dma
{
  #![warn(experimental)]

  pub fn read_memory_address(memory_size: uint, memory_address: &str)
  -> &str
  {
    memory_address
  }
}
