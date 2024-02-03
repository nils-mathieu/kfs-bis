use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut, RangeBounds};

/// A fixed-capacity array-backed vector.
pub struct ArrayVec<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: u8,
}

impl<T, const N: usize> ArrayVec<T, N> {
    const _ENSURE_CAPACITY: () = assert!(N <= 255, "ArrayVec must have a non-zero capacity");

    /// Creates a new [`ArrayVec<T, N>`] instance.
    #[inline]
    pub const fn new() -> Self {
        Self {
            data: MaybeUninit::uninit_array(),
            len: 0,
        }
    }

    /// Returns the current length of the vector.
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns whether the vector is empty.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns whether the vector is full.
    #[inline(always)]
    pub const fn is_full(&self) -> bool {
        self.len() == N
    }

    /// Clears the vector, removing all values.
    #[inline]
    pub fn clear(&mut self) {
        while self.pop().is_some() {}
    }

    /// Returns the capacity of the vector.
    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Pops a value from the vector.
    ///
    /// If the vector is already empty, this function returns `None`.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        self.len -= 1;

        unsafe { Some(self.data.get_unchecked(self.len()).assume_init_read()) }
    }

    /// Pushes a value into the vector.
    ///
    /// # Safety
    ///
    /// The vector must not be full when calling this function.
    pub unsafe fn push_unchecked(&mut self, value: T) {
        let len = self.len();
        self.data.get_unchecked_mut(len).write(value);
        self.len += 1;
    }

    /// Attempts to push a value into the vector.
    ///
    /// # Errors
    ///
    /// If the vector is full, this function returns its input as an error.
    #[inline]
    pub fn try_push(&mut self, value: T) -> Result<(), T> {
        if self.is_full() {
            return Err(value);
        }

        unsafe {
            self.push_unchecked(value);
        }

        Ok(())
    }

    /// Attempts to push a value into the vector.
    ///
    /// # Panics
    ///
    /// If the vector is full, this function panics.
    #[inline]
    #[track_caller]
    pub fn push(&mut self, value: T) {
        self.try_push(value)
            .ok()
            .expect("attempted to push a value into an ArrayVec<T> that was full")
    }

    /// Inserts a value into the vector without checking whether it is full.
    ///
    /// # Safety
    ///
    /// The vector must not be full when calling this function. Additionally, the provided
    /// index is not checked for being out of bounds.
    pub unsafe fn insert_unchecked(&mut self, index: usize, value: T) {
        unsafe {
            // Copy the second part of the vector to make space for the new value.
            core::ptr::copy(
                self.data.as_ptr().add(index),
                self.data.as_mut_ptr().add(index + 1),
                self.len() - index,
            );

            // Write the new value to fill the created hole.
            self.data.get_unchecked_mut(index).write(value);

            self.len += 1;
        }
    }

    /// Attempts to insert a value into the vector.
    ///
    /// # Panics
    ///
    /// This function panics if `index` is out of bounds. It is allowed to be equal
    /// to the length of the vector, in which case the value is appended to the end.
    ///
    /// # Errors
    ///
    /// This function returns its input as an error if the vector is full.
    #[inline]
    #[track_caller]
    pub fn try_insert(&mut self, index: usize, value: T) -> Result<(), T> {
        assert!(index <= self.len(), "index out of bounds");

        if self.is_full() {
            return Err(value);
        }

        unsafe {
            self.insert_unchecked(index, value);
        }

        Ok(())
    }

    /// Removes an existing value from the vector.
    ///
    /// # Safety
    ///
    /// This function assumes that the provided index is within the bounds of the vector.
    pub unsafe fn remove_unchecked(&mut self, index: usize) -> T {
        let ret = self.data.get_unchecked(index).assume_init_read();

        // Fill the hole in the vector by moving the second part of the vector to the left.
        core::ptr::copy(
            self.data.as_ptr().add(index + 1),
            self.data.as_mut_ptr().add(index),
            self.len() - index - 1,
        );

        self.len -= 1;

        ret
    }

    /// Removes a range of values from the vector.
    ///
    /// This function is unchecked and assumes that the provided range is within the bounds of the
    /// vector.
    pub unsafe fn remove_range_unchecked(&mut self, start: usize, end: usize) {
        let count = end - start;

        // Drop the values that are being removed.
        let to_drop = core::slice::from_raw_parts_mut(self.data.as_mut_ptr().add(start), count);
        core::ptr::drop_in_place(to_drop);

        // Fill the hole in the vector by moving the second part of the vector to the left.
        core::ptr::copy(
            self.data.as_ptr().add(end),
            self.data.as_mut_ptr().add(start),
            self.len() - end,
        );

        self.len -= count as u8;
    }

    /// Removes a range of values from the vector.
    #[track_caller]
    pub fn remove_range(&mut self, range: impl RangeBounds<usize>) {
        use core::ops::Bound::*;

        let start = match range.start_bound() {
            Included(&start) => start,
            Excluded(&start) => start + 1,
            Unbounded => 0,
        };

        let end = match range.end_bound() {
            Included(&end) => end + 1,
            Excluded(&end) => end,
            Unbounded => self.len(),
        };

        assert!(
            start <= end,
            "range start must be less than or equal to range end"
        );

        assert!(
            end <= self.len(),
            "range end out of bounds ({}..{} > {})",
            start,
            end,
            self.len(),
        );

        unsafe {
            self.remove_range_unchecked(start, end);
        }
    }

    /// Extends the vector with elements from a slice.
    ///
    /// # Safety
    ///
    /// The vector must have enough capacity to hold the additional elements.
    pub unsafe fn extend_from_slice_unchecked(&mut self, slice: &[T])
    where
        T: Copy,
    {
        let len = self.len();
        core::ptr::copy_nonoverlapping(
            slice.as_ptr(),
            self.data.as_mut_ptr().add(len).cast(),
            slice.len(),
        );
        self.len += slice.len() as u8;
    }

    /// Extends the vector with elements from a slice.
    ///
    /// # Panics
    ///
    /// This function panics if the vector does not have enough capacity to hold the additional
    /// elements.
    pub fn extend_from_slice(&mut self, slice: &[T])
    where
        T: Copy,
    {
        assert!(
            self.len() + slice.len() <= self.capacity(),
            "ArrayVec::extend_from_slice: slice length exceeds capacity"
        );

        unsafe {
            self.extend_from_slice_unchecked(slice);
        }
    }
}

impl<T, const N: usize> Deref for ArrayVec<T, N> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.data.as_ptr() as *const T, self.len()) }
    }
}

impl<T, const N: usize> DerefMut for ArrayVec<T, N> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut T, self.len()) }
    }
}
