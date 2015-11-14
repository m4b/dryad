use auxv;
use utils::*;

const AUX_CNT:usize = 38;

//TODO: change *const to slices, esp. argv and envc
#[repr(C)]
pub struct KernelBlock {
    pub argc: isize,
    pub argv: *const *const u8,
    pub envc: isize,
    pub envp: *const *const u8,
    pub auxv: *const auxv::Elf64_auxv_t,
}

impl KernelBlock {
    pub fn getauxval(&self, t:u64) -> Result<u64, ()> {
        unsafe {
            let ptr = self.auxv.clone();
            let mut i = 1;
            let mut v = &*ptr;
            //            while v.a_type != auxv::AT::NULL {
            while v.a_type != 0 {
                if v.a_type == t {
                    return Ok (v.a_val);
                }
                v = &*ptr.offset(i);
                i += 1;
            }        
        }
        Err(())
    }
    
    pub fn new (args: *const u64) -> KernelBlock {        
        unsafe {
            let argc = (*args) as isize;
            let argv = args.offset(1) as *const *const u8;
            let envp = argv.offset(argc + 1);

            let mut p = envp;
            let mut envc = 1;
            // two null pointers mark end of envp
            // and beginning of the auxillary vectors
            while !(*p).is_null() {
                p = p.offset(1);
                envc += 1;
            }
            p = p.offset(1);
            let auxv = p as *const auxv::Elf64_auxv_t;
            KernelBlock{
                argc: argc,
                argv: argv,
                envc: envc,
                envp: envp,
                auxv: auxv,
            }
        }
    }

    pub fn get_aux (&self) -> Vec<u64> {
        let mut aux:Vec<u64> = vec![0; AUX_CNT];
        let mut i:isize = 0;
        unsafe {
            while (&*self.auxv.offset(i)).a_val != auxv::AT_NULL {
                let auxv_t = &*self.auxv.offset(i);
                aux[auxv_t.a_type as usize] = auxv_t.a_val;
                i += 1;
            }
        }
        aux        
    }

    pub unsafe fn unsafe_print (&self) -> () {
        write(&"argc: ");
        write_u64(self.argc as u64, false);
        write(&"\n");
        write(&"argv[0]: ");
        write_chars_at(*self.argv, 0);
        write(&"\n");
        write(&"envc: ");
        write_u64(self.envc as u64, false);
        write(&"\n");
    }
}
