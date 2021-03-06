//! `Forest` composed of disjoint `Tree`s.

use super::{Node,Link,Tree,Iter,IterMut,OntoIter,Size};
use super::bfs::{BfsForest,Splitted};
use crate::rust::*;

/// A nullable forest
pub struct Forest<T> {
    pub(crate) link : Link,
               mark : super::heap::Phantom<T>,
}

impl<T> Deref for Forest<T> {
    type Target = Link;
    fn deref( &self ) -> &Link { &self.link }
}

impl<T> Forest<T> {
    /// Makes an empty `Forest`
    #[inline] pub fn new() -> Forest<T> { Self::from( null_mut() )}

    /// Returns `true` if the `Forest` is empty.
    ///
    /// This operation should compute in O(1) time.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::linked::singly::{tr,fr};
    /// let mut forest = fr();
    /// assert!( forest.is_empty() );
    /// forest.push_back( tr(1) ); 
    /// assert!( !forest.is_empty() );
    /// ```
    #[inline] pub fn is_empty( &self ) -> bool { self.link.is_leaf() }

    #[inline] pub(crate) fn from( child: *mut Link ) -> Self {
        Forest{
            link: Link{ next: null_mut(), child },
            mark: PhantomData
        }
    }

    #[inline] pub(crate) fn clear( &mut self ) { self.link.reset_child(); }

    #[inline] pub(crate) unsafe fn set_sib( &mut self, sib: *mut Link ) { (*self.tail()).set_sib( sib ); }

    /// Returns the first child of the forest,
    /// or None if it is empty.
    pub fn first( &self ) -> Option<&Node<T>> {
        if self.is_empty() {
            None
        } else {
            unsafe { Some( &*( self.head() as *mut Node<T> ))}
        }
    }

    /// Returns a mutable pointer to the first child of the forest,
    /// or None if it is empty.
    pub fn first_mut( &mut self ) -> Option<Pin<&mut Node<T>>> {
        if self.is_empty() {
            None
        } else {
            unsafe { Some( Pin::new_unchecked( &mut *( self.head() as *mut Node<T> )))}
        }
    }

    /// Returns the last child of the forest,
    /// or None if it is empty.
    pub fn last( &self ) -> Option<&Node<T>> {
        if self.is_empty() {
            None
        } else {
            unsafe { Some( &*( self.tail() as *mut Node<T> ))}
        }
    }

    /// Returns a mutable pointer to the last child of the forest,
    /// or None if it is empty.
    pub fn last_mut( &mut self ) -> Option<Pin<&mut Node<T>>> {
        if self.is_empty() {
            None
        } else {
            unsafe { Some( Pin::new_unchecked( &mut *( self.tail() as *mut Node<T> )))}
        }
    }

    /// Adds the tree as the first child.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::linked::singly::{tr,fr};
    /// let mut forest = fr();
    /// forest.push_front( tr(1) );
    /// assert_eq!( forest.to_string(), "( 1 )" );
    /// forest.push_front( tr(2) );
    /// assert_eq!( forest.to_string(), "( 2 1 )" );
    /// ```
    #[inline] pub fn push_front( &mut self, mut tree: Tree<T> ) {
        let tree_root = tree.root_mut_().plink();
        if self.is_empty() {
            self.link.set_child( tree_root );
        } else { unsafe {
            tree.link_mut().set_sib( self.head() );
            self.link.adopt( tree_root );
        }}
        tree.clear();
    }

    /// Adds the tree as the first child.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::linked::singly::{tr,fr};
    /// let mut forest = fr();
    /// forest.push_back( tr(1) );
    /// assert_eq!( forest.to_string(), "( 1 )" );
    /// forest.push_back( tr(2) );
    /// assert_eq!( forest.to_string(), "( 1 2 )" );
    /// ```
    #[inline] pub fn push_back( &mut self, mut tree: Tree<T> ) {
        let tree_root = tree.root_mut_().plink();
        if !self.is_empty() {
            unsafe {
                tree.link_mut().set_sib( self.head() );
                self.link.adopt( tree_root );
            }
        }
        self.link.set_child( tree_root );
        tree.clear();
    }

    /// remove and return the first child
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::linked::singly::tr;
    /// let mut forest = -tr(1)-tr(2);
    /// assert_eq!( forest.pop_front(), Some( tr(1) ));
    /// assert_eq!( forest.to_string(), "( 2 )" );
    /// assert_eq!( forest.pop_front(), Some( tr(2) ));
    /// assert_eq!( forest.to_string(), "()" );
    /// ```
    #[inline] pub fn pop_front( &mut self ) -> Option<Tree<T>> {
        if self.is_empty() {
            None
        } else { unsafe {
            let front = self.head();
            if self.has_only_one_child() {
                self.clear();
            } else {
                (*self.tail()).set_sib( self.new_head() );
            }
            (*front).reset_sib();
            Some( Tree::from( front ))
        }}
    }

    /// merge the forest at front
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::linked::singly::{tr,fr};
    /// let mut forest = fr();
    /// forest.prepend( -tr(0)-tr(1) );
    /// assert_eq!( forest.to_string(), "( 0 1 )" );
    /// forest.prepend( -tr(2)-tr(3) );
    /// assert_eq!( forest.to_string(), "( 2 3 0 1 )" );
    /// ```
    #[inline] pub fn prepend( &mut self, mut forest: Forest<T> ) {
        if !forest.is_empty() {
            if self.is_empty() {
                self.link.set_child( forest.tail() );
            } else { unsafe {
                let forest_head = forest.head();
                forest.set_sib( self.head() );
                self.link.adopt( forest_head );
            }}
            forest.clear();
        }
    }

    /// merge the forest at back
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::linked::singly::{tr,fr};
    /// let mut forest = fr();
    /// forest.append( -tr(0)-tr(1) );
    /// assert_eq!( forest.to_string(), "( 0 1 )" );
    /// forest.append( -tr(2)-tr(3) );
    /// assert_eq!( forest.to_string(), "( 0 1 2 3 )" );
    /// ```
    #[inline] pub fn append( &mut self, mut forest: Forest<T> ) {
        if !forest.is_empty() {
            if !self.is_empty() { unsafe {
                let forest_head = forest.head();
                forest.set_sib( self.head() );
                self.link.adopt( forest_head );
            }}
            self.link.set_child( forest.tail() );
            forest.clear();
        }
    }

    /// Provides a forward iterator over child `Node`s
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::linked::singly::{tr,fr};
    ///
    /// let forest = fr::<i32>();
    /// assert_eq!( forest.iter().next(), None );
    ///
    /// let forest = -tr(1)-tr(2);
    /// let mut iter = forest.iter();
    /// assert_eq!( iter.next(), Some( tr(1).root() ));
    /// assert_eq!( iter.next(), Some( tr(2).root() ));
    /// assert_eq!( iter.next(), None );
    /// assert_eq!( iter.next(), None );
    /// ```
    #[inline] pub fn iter<'a>( &self ) -> Iter<'a,T> {
        if self.is_empty() {
            Iter::new( null(), null() )
        } else { unsafe {
            Iter::new( self.head(), self.tail() )
        }}
    }

    /// Provides a forward iterator over child `Node`s with mutable references.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::linked::singly::{tr,fr};
    ///
    /// let mut forest = fr::<i32>();
    /// assert_eq!( forest.iter_mut().next(), None );
    ///
    /// let mut forest = -tr(1)-tr(2);
    /// for mut child in forest.iter_mut() { child.data *= 10; }
    /// assert_eq!( forest.to_string(), "( 10 20 )" );
    /// ```
    #[inline] pub fn iter_mut<'a>( &mut self ) -> IterMut<'a,T> {
        if self.is_empty() {
            IterMut::new( null_mut(), null_mut() )
        } else { unsafe {
            IterMut::new( self.head(), self.tail() )
        }}
    }

    /// Provide an iterator over `Forest`'s `Subnode`s for insert/remove at any position.
    /// See `Subnode`'s document for more.
    #[inline] pub fn onto_iter<'a>( &mut self ) -> OntoIter<'a,T> {
        unsafe {
            if self.is_empty() {
                OntoIter {
                    next: null_mut(), curr: null_mut(), prev: null_mut(), child: null_mut(),
                    parent: &mut self.link as *mut Link,
                    mark: PhantomData,
                }
            } else {
                OntoIter {
                    next   : self.head(),
                    curr   : null_mut(),
                    prev   : self.tail(),
                    child  : self.tail(),
                    parent : &mut self.link as *mut Link,
                    mark   : PhantomData,
                }
            }
        }
    }

    /// Provides a forward iterator in a breadth-first manner
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{bfs,Size};
    /// use trees::linked::singly::{tr,fr};
    ///
    /// let forest = fr::<i32>();
    /// let visits = forest.bfs().iter.collect::<Vec<_>>();
    /// assert!( visits.is_empty() );
    ///
    /// let forest = -( tr(1)/tr(2)/tr(3) ) -( tr(4)/tr(5)/tr(6) );
    /// let visits = forest.bfs().iter.collect::<Vec<_>>();
    /// assert_eq!( visits, vec![
    ///     bfs::Visit{ data: &1, size: Size{ degree: 2, node_cnt: 0 }},
    ///     bfs::Visit{ data: &4, size: Size{ degree: 2, node_cnt: 0 }},
    ///     bfs::Visit{ data: &2, size: Size{ degree: 0, node_cnt: 0 }},
    ///     bfs::Visit{ data: &3, size: Size{ degree: 0, node_cnt: 0 }},
    ///     bfs::Visit{ data: &5, size: Size{ degree: 0, node_cnt: 0 }},
    ///     bfs::Visit{ data: &6, size: Size{ degree: 0, node_cnt: 0 }},
    /// ]);
    /// ```
    pub fn bfs<'a, 's:'a>( &'s self ) -> BfsForest<Splitted<Iter<'a,T>>> {
        let size = Size{ degree: self.iter().count() as u32, node_cnt: 0 };
        let mut iters = VecDeque::new();
        iters.push_back( self.iter() );
        let iter = Splitted{ iters };
        BfsForest{ iter, size }
    }

    /// Provides a forward iterator with mutable references in a breadth-first manner
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{bfs,Size};
    /// use trees::linked::singly::{tr,fr};
    ///
    /// let mut forest = fr::<i32>();
    /// let visits = forest.bfs_mut().iter.collect::<Vec<_>>();
    /// assert!( visits.is_empty() );
    ///
    /// let mut forest = -( tr(1)/tr(2)/tr(3) ) -( tr(4)/tr(5)/tr(6) );
    /// let visits = forest.bfs_mut().iter.collect::<Vec<_>>();
    /// assert_eq!( visits, vec![
    ///     bfs::Visit{ data: &mut 1, size: Size{ degree: 2, node_cnt: 0 }},
    ///     bfs::Visit{ data: &mut 4, size: Size{ degree: 2, node_cnt: 0 }},
    ///     bfs::Visit{ data: &mut 2, size: Size{ degree: 0, node_cnt: 0 }},
    ///     bfs::Visit{ data: &mut 3, size: Size{ degree: 0, node_cnt: 0 }},
    ///     bfs::Visit{ data: &mut 5, size: Size{ degree: 0, node_cnt: 0 }},
    ///     bfs::Visit{ data: &mut 6, size: Size{ degree: 0, node_cnt: 0 }},
    /// ]);
    /// ```
    pub fn bfs_mut<'a, 's:'a>( &'s mut self ) -> BfsForest<Splitted<IterMut<'a,T>>> {
        let size = Size{ degree: self.iter().count() as u32, node_cnt: 0 };
        let mut iters = VecDeque::new();
        iters.push_back( self.iter_mut() );
        let iter = Splitted{ iters };
        BfsForest{ iter, size }
    }

    /// Provides a forward iterator with owned data in a breadth-first manner
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{bfs,Size};
    /// use trees::linked::singly::{tr,fr};
    ///
    /// let forest = fr::<i32>();
    /// let visits = forest.into_bfs().iter.collect::<Vec<_>>();
    /// assert!( visits.is_empty() );
    ///
    /// let forest = -( tr(1)/tr(2)/tr(3) ) -( tr(4)/tr(5)/tr(6) );
    /// let visits = forest.into_bfs().iter.collect::<Vec<_>>();
    /// assert_eq!( visits, vec![
    ///     bfs::Visit{ data: 1, size: Size{ degree: 2, node_cnt: 0 }},
    ///     bfs::Visit{ data: 4, size: Size{ degree: 2, node_cnt: 0 }},
    ///     bfs::Visit{ data: 2, size: Size{ degree: 0, node_cnt: 0 }},
    ///     bfs::Visit{ data: 3, size: Size{ degree: 0, node_cnt: 0 }},
    ///     bfs::Visit{ data: 5, size: Size{ degree: 0, node_cnt: 0 }},
    ///     bfs::Visit{ data: 6, size: Size{ degree: 0, node_cnt: 0 }},
    /// ]);
    /// ```
    pub fn into_bfs( self ) -> BfsForest<Splitted<IntoIter<T>>> {
        let size = Size{ degree: self.iter().count() as u32, node_cnt: 0 };
        BfsForest::from( self, size )
    }
}

impl<T:Clone> Clone for Forest<T> {
    fn clone( &self ) -> Self {
        let mut forest = Forest::<T>::new();
        for child in self.iter() {
            forest.push_back( child.to_owned() );
        }
        forest
    }
}

impl<T> Default for Forest<T> { #[inline] fn default() -> Self { Self::new() }}

impl<T> Drop for Forest<T> {
    fn drop( &mut self ) {
        while let Some(_) = self.pop_front() {}
    }
}

pub struct IntoIter<T> {
    pub(crate) forest : Forest<T>,
    pub(crate) marker : PhantomData<Tree<T>>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = Tree<T>;

    #[inline] fn next( &mut self ) -> Option<Tree<T>> { self.forest.pop_front() }
    #[inline] fn size_hint( &self ) -> ( usize, Option<usize> ) { self.forest.iter().size_hint() }
}

impl<T> ExactSizeIterator for IntoIter<T> {}

impl<T> Drop for IntoIter<T> {
    fn drop( &mut self ) {
        for _ in self.by_ref() {}
    }
}

impl<T> IntoIterator for Forest<T> {
    type Item = Tree<T>;
    type IntoIter = IntoIter<T>;

    #[inline] fn into_iter( self ) -> IntoIter<T> { IntoIter{ forest: self, marker: PhantomData }}
}

impl<T> FromIterator<Tree<T>> for Forest<T> {
   fn from_iter<I:IntoIterator<Item=Tree<T>>>( iter: I ) -> Self {
        let mut iter = iter.into_iter();
        let mut children = Forest::<T>::new();
        while let Some( node ) = iter.next() {
            children.push_back( node );
        }
        children
    }
}

impl<T> Extend<Tree<T>> for Forest<T> {
    fn extend<I:IntoIterator<Item=Tree<T>>>( &mut self, iter: I ) {
        for child in iter.into_iter() {
            self.push_back( child );
        }
    }
}

impl<T:Debug> Debug for Forest<T> { fn fmt( &self, f: &mut Formatter ) -> fmt::Result {
    if self.is_empty() {
            write!( f, "()" )
        } else {
            write!( f, "( " )?;
            for child in self.iter() {
                write!( f, "{:?} ", child )?;
            }
            write!( f, ")" )
        }
    }
}

impl<T:Display> Display for Forest<T> {
    fn fmt( &self, f: &mut Formatter ) -> fmt::Result {
        if self.is_empty() {
            write!( f, "()" )
        } else {
            write!( f, "( " )?;
            for child in self.iter() {
                write!( f, "{} ", child )?;
            }
            write!( f, ")" )
        }
    }
}

impl<T:PartialEq> PartialEq for Forest<T> {
    fn eq( &self, other: &Self ) -> bool { self.iter().eq( other.iter() )}
    fn ne( &self, other: &Self ) -> bool { self.iter().ne( other.iter() )}
}

impl<T:Eq> Eq for Forest<T> {}

impl<T:PartialOrd> PartialOrd for Forest<T> {
    fn partial_cmp( &self, other: &Self ) -> Option<Ordering> {
        self.iter().partial_cmp( other.iter() )
    }
}

impl<T:Ord> Ord for Forest<T> {
    #[inline] fn cmp( &self, other: &Self ) -> Ordering {
        self.iter().cmp( other.iter() )
    }
}

impl<T:Hash> Hash for Forest<T> {
    fn hash<H:Hasher>( &self, state: &mut H ) {
        for child in self.iter() {
            child.hash( state );
        }
    }
}

unsafe impl<T:Send> Send for Forest<T> {}
unsafe impl<T:Sync> Sync for Forest<T> {}
