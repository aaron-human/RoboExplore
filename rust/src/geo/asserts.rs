
/// Asserts that two values are very close (i.e. floats usually don't exactly match).
#[macro_export]
macro_rules! assert_about_eq {
	( $left:expr , $right:expr ) => {
		let left = $left;
		let right = $right;
		let delta = (left - right).abs();
		if delta > EPSILON {
			panic!("assertion failed: (left == right)\nleft: {:?}\nright: {:?}\ndelta: {:?} > {:?}", left, right, delta, EPSILON);
		}
	};
	( $left:expr , $right:expr , $($message : tt)+ ) => {
		let left = $left;
		let right = $right;
		let delta = (left - right).abs();
		if delta > EPSILON {
			panic!("assertion failed: (left == right)\nleft: {:?}\nright: {:?}\ndelta: {:?} > {:?}\nMessage: {}", left, right, delta, EPSILON, format_args!( $($message : tt)+ ));
		}
	};
}

/// Asserts that two Vec2 instances are very close (within EPSILON distance).
#[macro_export]
macro_rules! assert_vec2_about_eq {
	( $left:expr , $right:expr ) => {
		let left = $left;
		let right = $right;
		let delta = (&left - &right).length();
		if delta > EPSILON {
			panic!("assertion failed: (left == right)\nleft: {:?}\nright: {:?}\ndistance: {:?} > {:?}", left, right, delta, EPSILON);
		}
	};
	( $left:expr , $right:expr , $($message : tt)+ ) => {
		let left = $left;
		let right = $right;
		let delta = (&left - &right).length();
		if delta > EPSILON {
			panic!("assertion failed: (left == right)\nleft: {:?}\nright: {:?}\ndistance: {:?} > {:?}\nMessage: {}", left, right, delta, EPSILON, format_args!( $($message : tt)+ ));
		}
	};
}

/// Asserts that one value is less than another.
#[macro_export]
macro_rules! assert_lt {
	( $left:expr , $right:expr ) => {
		let left = $left;
		let right = $right;
		if left >= right {
			panic!("assertion failed: (left < right)\nleft: {:?}\nright: {:?}", left, right);
		}
	};
	( $left:expr , $right:expr , $($message : tt)+ ) => {
		let left = $left;
		let right = $right;
		if left >= right {
			panic!("assertion failed: (left < right)\nleft: {:?}\nright: {:?}\nMessage: {}", left, right, format_args!( $($message : tt)+ ));
		}
	};
}

/// Asserts that one value is greater than another.
#[macro_export]
macro_rules! assert_gt {
	( $left:expr , $right:expr ) => {
		let left = $left;
		let right = $right;
		if left <= right {
			panic!("assertion failed: (left > right)\nleft: {:?}\nright: {:?}", left, right);
		}
	};
	( $left:expr , $right:expr , $($message : tt)+ ) => {
		let left = $left;
		let right = $right;
		if left <= right {
			panic!("assertion failed: (left > right)\nleft: {:?}\nright: {:?}\nMessage: {}", left, right, format_args!( $($message : tt)+ ));
		}
	};
}
