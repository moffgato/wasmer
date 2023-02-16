use super::*;
use crate::syscalls::*;

pub fn fd_prestat_dir_name<M: MemorySize>(
    ctx: FunctionEnvMut<'_, WasiEnv>,
    fd: WasiFd,
    path: WasmPtr<u8, M>,
    path_len: M::Offset,
) -> Errno {
    trace!(
        "wasi[{}:{}]::fd_prestat_dir_name: fd={}, path_len={}",
        ctx.data().pid(),
        ctx.data().tid(),
        fd,
        path_len
    );
    let env = ctx.data();
    let (memory, mut state) = env.get_memory_and_wasi_state(&ctx, 0);
    let path_chars = wasi_try_mem!(path.slice(&memory, path_len));

    let inode = wasi_try!(state.fs.get_fd_inode(fd));

    // check inode-val.is_preopened?

    trace!("=> inode: {:?}", inode);
    let guard = inode.read();
    match guard.deref() {
        Kind::Dir { .. } | Kind::Root { .. } => {
            // TODO: verify this: null termination, etc
            let path_len: u64 = path_len.into();
            if (inode.name.len() as u64) < path_len {
                wasi_try_mem!(path_chars
                    .subslice(0..inode.name.len() as u64)
                    .write_slice(inode.name.as_bytes()));
                wasi_try_mem!(path_chars.index(inode.name.len() as u64).write(0));

                //trace!("=> result: \"{}\"", inode_val.name);

                Errno::Success
            } else {
                Errno::Overflow
            }
        }
        Kind::Symlink { .. }
        | Kind::Buffer { .. }
        | Kind::File { .. }
        | Kind::Socket { .. }
        | Kind::Pipe { .. }
        | Kind::EventNotifications { .. } => Errno::Notdir,
    }
}
