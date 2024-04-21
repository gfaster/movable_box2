use std::{alloc::{Layout, self}, cell::Cell, mem::{offset_of, MaybeUninit}, ptr::{self, NonNull}};

struct MovableAllocation<T: Unpin> {
    fwd: Cell<*mut MovableAllocation<T>>,
    data: MaybeUninit<T>,
}

pub struct MoveBox<T: Unpin> {
    data: Cell<NonNull<MovableAllocation<T>>>,
}

impl<T: Unpin> MoveBox<T> {
    #[inline(never)]
    fn update_ptr(&self) -> *mut MovableAllocation<T> {
        let ptr = self.data.get().as_ptr();
        let fwd = unsafe { (*ptr).fwd.get() };
        if fwd == ptr {
            return fwd;
        } 
        unsafe { ptr::copy_nonoverlapping(&(*ptr).data, &mut (*fwd).data, 1) };
        unsafe { alloc::dealloc(ptr.cast(), Layout::new::<MovableAllocation<T>>()) };
        self.data.set(unsafe { NonNull::new_unchecked(fwd) });
        fwd
    }

    pub fn relocate(&self) {
        unsafe { MovableAllocation::relocate(self.data.get().as_ptr()) };
    }

    pub fn new(item: T) -> Self {
        item.into()
    }
}

impl<T: Unpin> std::ops::Deref for MoveBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let mut ptr = self.data.get().as_ptr();
        let fwd = unsafe { (*ptr).fwd.get() };
        if ptr != fwd {
            ptr = self.update_ptr();
        } 
        unsafe { (*ptr).data.assume_init_ref() }
    }
}

impl<T: Unpin> std::ops::DerefMut for MoveBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let mut ptr = self.data.get().as_ptr();
        let fwd = unsafe { (*ptr).fwd.get() };
        if ptr != fwd {
            ptr = self.update_ptr();
        } 
        unsafe { (*ptr).data.assume_init_mut() }

    }
}

impl<T: Unpin> Drop for MoveBox<T> {
    fn drop(&mut self) {
        let inner: *mut T = &mut **self;
        unsafe { inner.drop_in_place() };
        unsafe { alloc::dealloc(self.data.get().as_ptr().cast(), Layout::new::<MovableAllocation<T>>()) };
    }
}

impl<T: Unpin> From<T> for MoveBox<T> {
    fn from(value: T) -> Self {
        let bx = Box::new(MovableAllocation {
            fwd: Cell::new(ptr::null_mut()),
            data: MaybeUninit::new(value),
        });
        let p: *const MovableAllocation<T> = &*bx;
        bx.fwd.set(p.cast_mut());
        MoveBox { data: unsafe { Cell::new(NonNull::new_unchecked(Box::into_raw(bx))) } }
    }
}

impl<T: Unpin> MovableAllocation<T> {
    pub unsafe fn relocate(ptr: *const Self) {
        assert_eq!(ptr, unsafe { (*ptr).fwd.get() });
        let new = unsafe { alloc::alloc(Layout::new::<Self>()) };
        let fwd_p: *mut Cell<*mut Self> = unsafe { new.byte_add(offset_of!(Self, fwd)).cast() };
        unsafe { fwd_p.write(Cell::new(new.cast())) };
        unsafe { (*ptr).fwd.set(new.cast()) };
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let bx1 = MoveBox::new(5);
        assert_eq!(*bx1, 5);
        bx1.relocate();
        assert_eq!(*bx1, 5);
        let bx2 = MoveBox::new(6);
        assert_eq!(*bx2, 6);
        bx1.relocate();
        bx2.relocate();
        assert_eq!(*bx1, 5);
        assert_eq!(*bx2, 6);
    }
}
