use {
    crate::{
        bytemuck::{pod_from_bytes_mut, pod_slice_from_bytes_mut},
        error::PodSliceError,
        pod_length::PodLength,
        primitives::PodU64,
    },
    bytemuck::Pod,
    core::mem::{align_of, size_of},
    solana_program_error::ProgramError,
};

/// Calculate padding needed between types for alignment
#[inline]
fn calculate_padding<L: Pod, T: Pod>() -> Result<usize, ProgramError> {
    let length_size = size_of::<L>();
    let data_align = align_of::<T>();

    // Calculate how many bytes we need to add to length_size
    // to make it a multiple of data_align
    let remainder = length_size
        .checked_rem(data_align)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    if remainder == 0 {
        Ok(0)
    } else {
        data_align
            .checked_sub(remainder)
            .ok_or(ProgramError::ArithmeticOverflow)
    }
}

/// An API for interpreting a raw buffer (`&[u8]`) as a mutable, variable-length collection of Pod elements.
///
/// `ListView` provides a safe, zero-copy, `Vec`-like interface for a slice of
/// `Pod` data that resides in an external, pre-allocated `&mut [u8]` buffer.
/// It does not own the buffer itself, but acts as a mutable view over it.
///
/// This is useful in environments where allocations are restricted or expensive,
/// such as Solana programs, allowing for efficient reads and manipulation of
/// dynamic-length data structures.
///
/// ## Memory Layout
///
/// The structure assumes the underlying byte buffer is formatted as follows:
/// 1.  **Length**: A length field of type `L` at the beginning of the buffer,
///     indicating the number of currently active elements in the collection.  Defaults to `PodU64` so the offset is then compatible with 1, 2, 4 and 8 bytes.
/// 2.  **Padding**: Optional padding bytes to ensure proper alignment of the data.
/// 3.  **Data**: The remaining part of the buffer, which is treated as a slice
///     of `T` elements. The capacity of the collection is the number of `T`
///     elements that can fit into this data portion.
pub struct ListView<'data, T: Pod, L: PodLength = PodU64> {
    length: &'data mut L,
    data: &'data mut [T],
    max_length: usize,
}

impl<'data, T: Pod, L: PodLength> ListView<'data, T, L> {
    /// Unpack the mutable buffer into a mutable slice, with the option to
    /// initialize the data
    #[inline(always)]
    fn unpack_internal(buf: &'data mut [u8], init: bool) -> Result<Self, ProgramError> {
        // Split the buffer to get the length prefix.
        // buf: [ L L L L | P P D D D D D D D D ...]
        //       <-------> <---------------------->
        //       len_bytes          tail
        let length_size = size_of::<L>();
        if buf.len() < length_size {
            return Err(PodSliceError::BufferTooSmall.into());
        }
        let (len_bytes, tail) = buf.split_at_mut(length_size);

        // Skip alignment padding to find the start of the data.
        // tail: [P P | D D D D D D D D ...]
        //        <-> <------------------->
        //      padding    data_bytes
        let padding = calculate_padding::<L, T>()?;
        let data_bytes = tail
            .get_mut(padding..)
            .ok_or(PodSliceError::BufferTooSmall)?;

        // Cast the bytes to typed data
        let length = pod_from_bytes_mut::<L>(len_bytes)?;
        let data = pod_slice_from_bytes_mut::<T>(data_bytes)?;
        let max_length = data.len();

        // Initialize the list or validate its current length.
        if init {
            *length = L::try_from(0)?;
        } else if (*length).into() > max_length {
            return Err(PodSliceError::BufferTooSmall.into());
        }

        Ok(Self {
            length,
            data,
            max_length,
        })
    }

    /// Unpack the mutable buffer into a mutable slice
    pub fn unpack<'a>(data: &'a mut [u8]) -> Result<Self, ProgramError>
    where
        'a: 'data,
    {
        Self::unpack_internal(data, false)
    }

    /// Unpack the mutable buffer into a mutable slice, and initialize the
    /// slice to 0-length
    pub fn init<'a>(data: &'a mut [u8]) -> Result<Self, ProgramError>
    where
        'a: 'data,
    {
        Self::unpack_internal(data, true)
    }

    /// Add another item to the slice
    pub fn push(&mut self, item: T) -> Result<(), ProgramError> {
        let length = (*self.length).into();
        if length >= self.max_length {
            Err(PodSliceError::BufferTooSmall.into())
        } else {
            self.data[length] = item;
            *self.length = L::try_from(length.saturating_add(1))?;
            Ok(())
        }
    }

    /// Remove and return the element at `index`, shifting all later
    /// elements one position to the left.
    pub fn remove(&mut self, index: usize) -> Result<T, ProgramError> {
        let len = (*self.length).into();
        if index >= len {
            return Err(ProgramError::InvalidArgument);
        }

        let removed_item = self.data[index];

        // Move the tail left by one
        let tail_start = index
            .checked_add(1)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        self.data.copy_within(tail_start..len, index);

        // Zero-fill the now-unused slot at the end
        let last = len.saturating_sub(1);
        self.data[last] = T::zeroed();

        // Store the new length (len - 1)
        *self.length = L::try_from(last)?;

        Ok(removed_item)
    }

    /// Get the amount of bytes used by `num_items`
    pub fn size_of(num_items: usize) -> Result<usize, ProgramError> {
        let padding_size = calculate_padding::<L, T>()?;
        let header_size = size_of::<L>().saturating_add(padding_size);

        let data_size = size_of::<T>()
            .checked_mul(num_items)
            .ok_or(PodSliceError::CalculationFailure)?;

        header_size
            .checked_add(data_size)
            .ok_or(PodSliceError::CalculationFailure.into())
    }

    /// Get the current number of items in collection
    pub fn len(&self) -> usize {
        (*self.length).into()
    }

    /// Returns true if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the current elements
    pub fn iter(&self) -> std::slice::Iter<T> {
        let len = (*self.length).into();
        self.data[..len].iter()
    }

    /// Returns a mutable iterator over the current elements
    pub fn iter_mut(&mut self) -> std::slice::IterMut<T> {
        let len = (*self.length).into();
        self.data[..len].iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::primitives::{PodU16, PodU32, PodU64},
        bytemuck_derive::{Pod, Zeroable},
    };

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
    struct TestStruct {
        test_field: u8,
        test_pubkey: [u8; 32],
    }

    #[test]
    fn init_and_push() {
        let size = ListView::<TestStruct>::size_of(2).unwrap();
        let mut buffer = vec![0u8; size];

        let mut pod_slice = ListView::<TestStruct>::init(&mut buffer).unwrap();

        pod_slice.push(TestStruct::default()).unwrap();
        assert_eq!(*pod_slice.length, PodU64::from(1));
        assert_eq!(pod_slice.len(), 1);

        pod_slice.push(TestStruct::default()).unwrap();
        assert_eq!(*pod_slice.length, PodU64::from(2));
        assert_eq!(pod_slice.len(), 2);

        // Buffer should be full now
        let err = pod_slice.push(TestStruct::default()).unwrap_err();
        assert_eq!(err, PodSliceError::BufferTooSmall.into());
    }

    fn make_buffer<L: Pod + Into<usize> + TryFrom<usize>>(capacity: usize, items: &[u8]) -> Vec<u8>
    where
        PodSliceError: From<<L as TryFrom<usize>>::Error>,
        <L as TryFrom<usize>>::Error: std::fmt::Debug,
    {
        let length_size = size_of::<L>();
        let padding_size = calculate_padding::<L, u8>().unwrap();
        let header_size = length_size.saturating_add(padding_size);
        let buff_len = header_size.checked_add(capacity).unwrap();
        let mut buf = vec![0u8; buff_len];

        // Write the length
        let length = L::try_from(items.len()).unwrap();
        let length_bytes = bytemuck::bytes_of(&length);
        buf[..length_size].copy_from_slice(length_bytes);

        // Copy the data after the header
        let data_end = header_size.checked_add(items.len()).unwrap();
        buf[header_size..data_end].copy_from_slice(items);
        buf
    }

    #[test]
    fn remove_at_first_item() {
        let mut buff = make_buffer::<PodU64>(15, &[10, 20, 30, 40]);
        let mut list_view = ListView::<u8>::unpack(&mut buff).unwrap();
        let removed = list_view.remove(0).unwrap();
        assert_eq!(removed, 10);
        assert_eq!(list_view.len(), 3);
        assert_eq!(list_view.data[..list_view.len()].to_vec(), &[20, 30, 40]);
        assert_eq!(list_view.data[3], 0);
    }

    #[test]
    fn remove_at_middle_item() {
        let mut buff = make_buffer::<PodU64>(15, &[10, 20, 30, 40]);
        let mut list_view = ListView::<u8>::unpack(&mut buff).unwrap();
        let removed = list_view.remove(2).unwrap();
        assert_eq!(removed, 30);
        assert_eq!(list_view.len(), 3);
        assert_eq!(list_view.data[..list_view.len()].to_vec(), &[10, 20, 40]);
        assert_eq!(list_view.data[3], 0);
    }

    #[test]
    fn remove_at_last_item() {
        let mut buff = make_buffer::<PodU64>(15, &[10, 20, 30, 40]);
        let mut list_view = ListView::<u8>::unpack(&mut buff).unwrap();
        let removed = list_view.remove(3).unwrap();
        assert_eq!(removed, 40);
        assert_eq!(list_view.len(), 3);
        assert_eq!(list_view.data[..list_view.len()].to_vec(), &[10, 20, 30]);
        assert_eq!(list_view.data[3], 0);
    }

    #[test]
    fn remove_at_out_of_bounds() {
        let mut buff = make_buffer::<PodU64>(3, &[1, 2, 3]);
        let original_buff = buff.clone();

        let mut list_view = ListView::<u8>::unpack(&mut buff).unwrap();
        let err = list_view.remove(3).unwrap_err();
        assert_eq!(err, ProgramError::InvalidArgument);

        // list_view should be unchanged
        assert_eq!(list_view.len(), 3);
        assert_eq!(list_view.data[..list_view.len()].to_vec(), vec![1, 2, 3]);

        assert_eq!(buff, original_buff);
    }

    #[test]
    fn remove_at_single_element() {
        let mut buff = make_buffer::<PodU64>(1, &[10]);
        let mut list_view = ListView::<u8>::unpack(&mut buff).unwrap();
        let removed = list_view.remove(0).unwrap();
        assert_eq!(removed, 10);
        assert_eq!(list_view.len(), 0);
        assert_eq!(list_view.data[..list_view.len()].to_vec(), &[] as &[u8]);
        assert_eq!(list_view.data[0], 0);
    }

    #[test]
    fn remove_at_empty_slice() {
        let mut buff = make_buffer::<PodU64>(0, &[]);
        let original_buff = buff.clone();

        let mut list_view = ListView::<u8>::unpack(&mut buff).unwrap();
        let err = list_view.remove(0).unwrap_err();
        assert_eq!(err, ProgramError::InvalidArgument);

        // Assert list state is unchanged
        assert_eq!(list_view.len(), 0);

        assert_eq!(buff, original_buff);
    }

    #[test]
    fn test_different_length_types() {
        // Test with u16 length
        let mut buff16 = make_buffer::<PodU16>(5, &[1, 2, 3]);
        let list16 = ListView::<u8, PodU16>::unpack(&mut buff16).unwrap();
        assert_eq!(list16.len(), 3);
        assert_eq!(list16.len(), 3);

        // Test with u32 length
        let mut buff32 = make_buffer::<PodU32>(5, &[4, 5, 6]);
        let list32 = ListView::<u8, PodU32>::unpack(&mut buff32).unwrap();
        assert_eq!(list32.len(), 3);
        assert_eq!(list32.len(), 3);

        // Test with u64 length
        let mut buff64 = make_buffer::<PodU64>(5, &[7, 8, 9]);
        let list64 = ListView::<u8, PodU64>::unpack(&mut buff64).unwrap();
        assert_eq!(list64.len(), 3);
        assert_eq!(list64.len(), 3);
    }

    #[test]
    fn test_calculate_padding() {
        // When length and data have same alignment, no padding needed
        assert_eq!(calculate_padding::<PodU16, u16>().unwrap(), 0);
        assert_eq!(calculate_padding::<PodU32, u32>().unwrap(), 0);
        assert_eq!(calculate_padding::<PodU64, u64>().unwrap(), 0);

        // When data alignment is smaller than or divides length size
        assert_eq!(calculate_padding::<PodU32, u8>().unwrap(), 0);
        assert_eq!(calculate_padding::<PodU32, u16>().unwrap(), 0);
        assert_eq!(calculate_padding::<PodU64, u8>().unwrap(), 0);
        assert_eq!(calculate_padding::<PodU64, u16>().unwrap(), 0);
        assert_eq!(calculate_padding::<PodU64, u32>().unwrap(), 0);

        // When padding is needed
        assert_eq!(calculate_padding::<PodU16, u32>().unwrap(), 2); // 2 + 2 = 4 (align to 4)
        assert_eq!(calculate_padding::<PodU16, u64>().unwrap(), 6); // 2 + 6 = 8 (align to 8)
        assert_eq!(calculate_padding::<PodU32, u64>().unwrap(), 4); // 4 + 4 = 8 (align to 8)

        // Test with custom aligned structs
        #[repr(C, align(8))]
        #[derive(Pod, Zeroable, Copy, Clone)]
        struct Align8 {
            _data: [u8; 8],
        }

        #[repr(C, align(16))]
        #[derive(Pod, Zeroable, Copy, Clone)]
        struct Align16 {
            _data: [u8; 16],
        }

        assert_eq!(calculate_padding::<PodU16, Align8>().unwrap(), 6); // 2 + 6 = 8
        assert_eq!(calculate_padding::<PodU32, Align8>().unwrap(), 4); // 4 + 4 = 8
        assert_eq!(calculate_padding::<PodU64, Align8>().unwrap(), 0); // 8 % 8 = 0

        assert_eq!(calculate_padding::<PodU16, Align16>().unwrap(), 14); // 2 + 14 = 16
        assert_eq!(calculate_padding::<PodU32, Align16>().unwrap(), 12); // 4 + 12 = 16
        assert_eq!(calculate_padding::<PodU64, Align16>().unwrap(), 8); // 8 + 8 = 16
    }

    #[test]
    fn test_alignment_in_practice() {
        // u32 length with u64 data - needs 4 bytes padding
        let size = ListView::<u64, PodU32>::size_of(2).unwrap();
        let mut buffer = vec![0u8; size];
        let list = ListView::<u64, PodU32>::init(&mut buffer).unwrap();

        // Check that data pointer is 8-byte aligned
        let data_ptr = list.data.as_ptr() as usize;
        assert_eq!(data_ptr % 8, 0);

        // u16 length with u64 data - needs 6 bytes padding
        let size = ListView::<u64, PodU16>::size_of(2).unwrap();
        let mut buffer = vec![0u8; size];
        let list = ListView::<u64, PodU16>::init(&mut buffer).unwrap();

        let data_ptr = list.data.as_ptr() as usize;
        assert_eq!(data_ptr % 8, 0);
    }

    #[test]
    fn test_length_too_large() {
        // Create a buffer with capacity for 2 items
        let capacity = 2;
        let length_size = size_of::<PodU32>();
        let padding_size = calculate_padding::<PodU32, u8>().unwrap();
        let header_size = length_size.saturating_add(padding_size);
        let buff_len = header_size.checked_add(capacity).unwrap();
        let mut buffer = vec![0u8; buff_len];

        // Manually write a length value that exceeds the capacity
        let invalid_length = PodU32::try_from(capacity + 1).unwrap();
        let length_bytes = bytemuck::bytes_of(&invalid_length);
        buffer[..length_size].copy_from_slice(length_bytes);

        // Attempting to unpack should return BufferTooSmall error
        match ListView::<u8, PodU32>::unpack(&mut buffer) {
            Err(err) => assert_eq!(err, PodSliceError::BufferTooSmall.into()),
            Ok(_) => panic!("Expected BufferTooSmall error, but unpack succeeded"),
        }
    }
}
