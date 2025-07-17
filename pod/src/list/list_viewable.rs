use {bytemuck::Pod, std::slice::Iter};

/// A trait to abstract the shared, read-only behavior
/// between `ListViewReadOnly` and `ListViewMut`.
pub trait ListViewable {
    /// The type of the items stored in the list.
    type Item: Pod;

    /// Returns the number of items in the list.
    fn len(&self) -> usize;

    /// Returns `true` if the list contains no items.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the total number of items that can be stored in the list.
    fn capacity(&self) -> usize;

    /// Returns a read-only slice of the items currently in the list.
    fn as_slice(&self) -> &[Self::Item];

    /// Returns a read-only iterator over the list.
    fn iter(&self) -> Iter<'_, Self::Item> {
        self.as_slice().iter()
    }
}
