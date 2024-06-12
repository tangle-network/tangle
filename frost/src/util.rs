use super::traits::{Ciphersuite, Element, Field, Group, Scalar};

pub fn scalar_is_valid<C: Ciphersuite>(scalar: &Scalar<C>) -> bool {
	*scalar != <<C::Group as Group>::Field as Field>::zero()
}

pub fn element_is_valid<C: Ciphersuite>(element: &Element<C>) -> bool {
	*element != C::Group::generator() && *element != C::Group::identity()
}
