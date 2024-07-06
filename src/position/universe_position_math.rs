use std::ops::{Add, Sub, AddAssign, SubAssign};
use super::universe_location::UniverseLocation;
use super::universe_transform::UniverseTransform;
use bevy::math::DVec3;

// UniverseLocation impls

impl<T> AddAssign<T> for UniverseLocation where T: Into<DVec3> {
    fn add_assign(&mut self, rhs: T) {
        self.position += rhs.into();
    } 
}

impl<T> Add<T> for UniverseLocation where T: Into<DVec3> {
    type Output = Self;
    fn add(mut self, rhs: T) -> Self {
        self += rhs;
        self
    } 
}

impl<T> SubAssign<T> for UniverseLocation where T: Into<DVec3> {
    fn sub_assign(&mut self, rhs: T) {
        self.position -= rhs.into();
    } 
}

impl<T> Sub<T> for UniverseLocation where T: Into<DVec3> {
    type Output = Self;
    fn sub(mut self, rhs: T) -> Self {
        self -= rhs;
        self
    } 
}

// UniverseTransform impls

impl<T> AddAssign<T> for UniverseTransform where T: Into<DVec3> {
    fn add_assign(&mut self, rhs: T) {
        self.loc += rhs.into();
    } 
}

impl<T> Add<T> for UniverseTransform where T: Into<DVec3> {
    type Output = Self;
    fn add(mut self, rhs: T) -> Self {
        self += rhs;
        self
    } 
}

impl<T> SubAssign<T> for UniverseTransform where T: Into<DVec3> {
    fn sub_assign(&mut self, rhs: T) {
        self.loc -= rhs.into();
    } 
}

impl<T> Sub<T> for UniverseTransform where T: Into<DVec3> {
    type Output = Self;
    fn sub(mut self, rhs: T) -> Self {
        self -= rhs;
        self
    } 
}