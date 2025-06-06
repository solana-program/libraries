use crate::bytemuck::{pod_from_bytes_mut, pod_slice_from_bytes_mut};
use crate::error::PodSliceError;
use crate::primitives::PodU32;
use crate::slice::max_len_for_type;
use bytemuck::Pod;
use solana_program_error::ProgramError;

const LENGTH_SIZE: usize = std::mem::size_of::<PodU32>();

/// A mutable, variable-length collection of `Pod` types backed by a byte buffer.
///
/// `PodList` provides a safe, zero-copy, `Vec`-like interface for a slice of
/// `Pod` data that resides in an external, pre-allocated `&mut [u8]` buffer.
/// It does not own the buffer itself, but acts as a mutable view over it.
///
/// This is useful in environments where allocations are restricted or expensive,
/// such as Solana programs, allowing for dynamic-length data structures within a
/// fixed-size account.
///
/// ## Memory Layout
///
/// The structure assumes the underlying byte buffer is formatted as follows:
/// 1.  **Length**: A `u32` value (`PodU32`) at the beginning of the buffer,
///     indicating the number of currently active elements in the collection.
/// 2.  **Data**: The remaining part of the buffer, which is treated as a slice
///     of `T` elements. The capacity of the collection is the number of `T`
///     elements that can fit into this data portion.
pub struct PodList<'data, T: Pod> {
    length: &'data mut PodU32,
    data: &'data mut [T],
    max_length: usize,
}

impl<'data, T: Pod> PodList<'data, T> {
    /// Unpack the mutable buffer into a mutable slice, with the option to
    /// initialize the data
    fn unpack_internal<'a>(data: &'a mut [u8], init: bool) -> Result<Self, ProgramError>
    where
        'a: 'data,
    {
        if data.len() < LENGTH_SIZE {
            return Err(PodSliceError::BufferTooSmall.into());
        }
        let (length, data) = data.split_at_mut(LENGTH_SIZE);
        let length = pod_from_bytes_mut::<PodU32>(length)?;
        if init {
            *length = 0.into();
        }
        let max_length = max_len_for_type::<T>(data.len(), u32::from(*length) as usize)?;
        let data = pod_slice_from_bytes_mut(data)?;
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
        Self::unpack_internal(data, /* init */ false)
    }

    /// Unpack the mutable buffer into a mutable slice, and initialize the
    /// slice to 0-length
    pub fn init<'a>(data: &'a mut [u8]) -> Result<Self, ProgramError>
    where
        'a: 'data,
    {
        Self::unpack_internal(data, /* init */ true)
    }

    /// Add another item to the slice
    pub fn push(&mut self, t: T) -> Result<(), ProgramError> {
        let length = u32::from(*self.length);
        if length as usize == self.max_length {
            Err(PodSliceError::BufferTooSmall.into())
        } else {
            self.data[length as usize] = t;
            *self.length = length.saturating_add(1).into();
            Ok(())
        }
    }

    /// Remove and return the element at `index`, shifting all later
    /// elements one position to the left.
    pub fn remove_at(&mut self, index: usize) -> Result<T, ProgramError> {
        let len = u32::from(*self.length) as usize;
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
        let last = len.checked_sub(1).ok_or(ProgramError::ArithmeticOverflow)?;
        self.data[last] = T::zeroed();

        // Store the new length (len - 1)
        *self.length = (last as u32).into();

        Ok(removed_item)
    }

    /// Find the first element that satisfies `predicate` and remove it,
    /// returning the element.
    pub fn remove_first_where<P>(&mut self, mut predicate: P) -> Result<T, ProgramError>
    where
        P: FnMut(&T) -> bool,
    {
        if let Some(index) = self.data.iter().position(&mut predicate) {
            self.remove_at(index)
        } else {
            Err(ProgramError::InvalidArgument)
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        bytemuck_derive::{Pod, Zeroable},
    };

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
    struct TestStruct {
        test_field: u8,
        test_pubkey: [u8; 32],
    }

    #[test]
    fn test_pod_collection() {
        // slice can fit 2 `TestStruct`
        let mut pod_slice_bytes = [0; 70];
        // set length to 1, so we have room to push 1 more item
        let len_bytes = [1, 0, 0, 0];
        pod_slice_bytes[0..4].copy_from_slice(&len_bytes);

        let mut pod_slice = PodList::<TestStruct>::unpack(&mut pod_slice_bytes).unwrap();

        assert_eq!(*pod_slice.length, PodU32::from(1));
        pod_slice.push(TestStruct::default()).unwrap();
        assert_eq!(*pod_slice.length, PodU32::from(2));
        let err = pod_slice
            .push(TestStruct::default())
            .expect_err("Expected an `PodSliceError::BufferTooSmall` error");
        assert_eq!(err, PodSliceError::BufferTooSmall.into());
    }

    fn make_buffer(capacity: usize, items: &[u8]) -> Vec<u8> {
        let buff_len = LENGTH_SIZE.checked_add(capacity).unwrap();
        let mut buf = vec![0u8; buff_len];
        buf[..LENGTH_SIZE].copy_from_slice(&(items.len() as u32).to_le_bytes());
        let end = LENGTH_SIZE.checked_add(items.len()).unwrap();
        buf[LENGTH_SIZE..end].copy_from_slice(items);
        buf
    }

    #[test]
    fn remove_at_first_item() {
        let mut buff = make_buffer(15, &[10, 20, 30, 40]);
        let mut pod_list = PodList::<u8>::unpack(&mut buff).unwrap();
        let removed = pod_list.remove_at(0).unwrap();
        assert_eq!(removed, 10);
        let pod_list_len = u32::from(*pod_list.length) as usize;
        assert_eq!(pod_list_len, 3);
        assert_eq!(pod_list.data[..pod_list_len].to_vec(), &[20, 30, 40]);
        assert_eq!(pod_list.data[3], 0);
    }

    #[test]
    fn remove_at_middle_item() {
        let mut buff = make_buffer(15, &[10, 20, 30, 40]);
        let mut pod_list = PodList::<u8>::unpack(&mut buff).unwrap();
        let removed = pod_list.remove_at(2).unwrap();
        assert_eq!(removed, 30);
        let pod_list_len = u32::from(*pod_list.length) as usize;
        assert_eq!(pod_list_len, 3);
        assert_eq!(pod_list.data[..pod_list_len].to_vec(), &[10, 20, 40]);
        assert_eq!(pod_list.data[3], 0);
    }

    #[test]
    fn remove_at_last_item() {
        let mut buff = make_buffer(15, &[10, 20, 30, 40]);
        let mut pod_list = PodList::<u8>::unpack(&mut buff).unwrap();
        let removed = pod_list.remove_at(3).unwrap();
        assert_eq!(removed, 40);
        let pod_list_len = u32::from(*pod_list.length) as usize;
        assert_eq!(pod_list_len, 3);
        assert_eq!(pod_list.data[..pod_list_len].to_vec(), &[10, 20, 30]);
        assert_eq!(pod_list.data[3], 0);
    }

    #[test]
    fn remove_at_out_of_bounds() {
        let mut buff = make_buffer(3, &[1, 2, 3]);
        let original_buff = buff.clone();

        {
            let mut pod_list = PodList::<u8>::unpack(&mut buff).unwrap();
            let err = pod_list.remove_at(3).unwrap_err();
            assert_eq!(err, ProgramError::InvalidArgument);

            // pod_list should be unchanged
            let pod_list_len = u32::from(*pod_list.length) as usize;
            assert_eq!(pod_list_len, 3);
            assert_eq!(pod_list.data[..pod_list_len].to_vec(), vec![1, 2, 3]);
        }

        assert_eq!(buff, original_buff);
    }

    #[test]
    fn remove_at_single_element() {
        let mut buff = make_buffer(1, &[10]);
        let mut pod_list = PodList::<u8>::unpack(&mut buff).unwrap();
        let removed = pod_list.remove_at(0).unwrap();
        assert_eq!(removed, 10);
        let pod_list_len = u32::from(*pod_list.length) as usize;
        assert_eq!(pod_list_len, 0);
        assert_eq!(pod_list.data[..pod_list_len].to_vec(), &[] as &[u8]);
        assert_eq!(pod_list.data[0], 0);
    }

    #[test]
    fn remove_at_empty_slice() {
        let mut buff = make_buffer(0, &[]);
        let original_buff = buff.clone();

        {
            let mut pod_list = PodList::<u8>::unpack(&mut buff).unwrap();
            let err = pod_list.remove_at(0).unwrap_err();
            assert_eq!(err, ProgramError::InvalidArgument);

            // Assert list state is unchanged
            let pod_list_len = u32::from(*pod_list.length) as usize;
            assert_eq!(pod_list_len, 0);
        }

        assert_eq!(buff, original_buff);
    }

    #[test]
    fn remove_first_where_first_item() {
        let mut buff = make_buffer(3, &[5, 10, 15]);
        let mut pod_list = PodList::<u8>::unpack(&mut buff).unwrap();
        let removed = pod_list.remove_first_where(|&x| x == 5).unwrap();
        assert_eq!(removed, 5);
        let pod_list_len = u32::from(*pod_list.length) as usize;
        assert_eq!(pod_list_len, 2);
        assert_eq!(pod_list.data[..pod_list_len].to_vec(), &[10, 15]);
        assert_eq!(pod_list.data[2], 0);
    }

    #[test]
    fn remove_first_where_middle_item() {
        let mut buff = make_buffer(4, &[1, 2, 3, 4]);
        let mut pod_list = PodList::<u8>::unpack(&mut buff).unwrap();
        let removed = pod_list.remove_first_where(|v| *v == 3).unwrap();
        assert_eq!(removed, 3);
        let pod_list_len = u32::from(*pod_list.length) as usize;
        assert_eq!(pod_list_len, 3);
        assert_eq!(pod_list.data[..pod_list_len].to_vec(), &[1, 2, 4]);
        assert_eq!(pod_list.data[3], 0);
    }

    #[test]
    fn remove_first_where_last_item() {
        let mut buff = make_buffer(3, &[5, 10, 15]);
        let mut pod_list = PodList::<u8>::unpack(&mut buff).unwrap();
        let removed = pod_list.remove_first_where(|&x| x == 15).unwrap();
        assert_eq!(removed, 15);
        let pod_list_len = u32::from(*pod_list.length) as usize;
        assert_eq!(pod_list_len, 2);
        assert_eq!(pod_list.data[..pod_list_len].to_vec(), &[5, 10]);
        assert_eq!(pod_list.data[2], 0);
    }

    #[test]
    fn remove_first_where_multiple_matches() {
        let mut buff = make_buffer(5, &[7, 8, 8, 9, 10]);
        let mut pod_list = PodList::<u8>::unpack(&mut buff).unwrap();
        let removed = pod_list.remove_first_where(|v| *v == 8).unwrap();
        assert_eq!(removed, 8); // Removed *first* 8
        let pod_list_len = u32::from(*pod_list.length) as usize;
        assert_eq!(pod_list_len, 4);
        // Should remove only the *first* match.
        assert_eq!(pod_list.data[..pod_list_len].to_vec(), &[7, 8, 9, 10]);
        assert_eq!(pod_list.data[4], 0);
    }

    #[test]
    fn remove_first_where_not_found() {
        let mut buff = make_buffer(3, &[5, 6, 7]);
        let original_buff = buff.clone();

        {
            let mut pod_list = PodList::<u8>::unpack(&mut buff).unwrap();
            let err = pod_list.remove_first_where(|v| *v == 42).unwrap_err();
            assert_eq!(err, ProgramError::InvalidArgument);
            // Assert list state is unchanged
            assert_eq!(u32::from(*pod_list.length) as usize, 3);
        }

        assert_eq!(buff, original_buff);
    }

    #[test]
    fn remove_first_where_empty_slice() {
        let mut buff = make_buffer(0, &[]);
        let original_buff = buff.clone();

        {
            let mut pod_list = PodList::<u8>::unpack(&mut buff).unwrap();
            let err = pod_list.remove_first_where(|_| true).unwrap_err();
            assert_eq!(err, ProgramError::InvalidArgument);
            // Assert list state is unchanged
            assert_eq!(u32::from(*pod_list.length) as usize, 0);
        }

        assert_eq!(buff, original_buff);
    }
}
