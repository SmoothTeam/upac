use super::ctypes::{CDiffKind, CSlice};
use super::Validate;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CDiffFileEntry {
    struct_size: usize,
    pub path: CSlice,
    pub kind: CDiffKind,
    pub package_name: CSlice,
    pub is_user: bool,
}

impl Validate for CDiffFileEntry {
    fn validate(&self) -> anyhow::Result<()> {
        if self.struct_size != size_of::<Self>() {
            return Err(anyhow::anyhow!("CDiffFileEntry: abi mismatch"));
        }
        self.path.validate()?;
        Ok(())
    }
}
