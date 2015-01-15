//#![feature(convert)]
use std::ops::{Add, Mul, Rem, BitXor, Not};
extern crate rand;
use std::io;
use std::io::Write;


struct Vector {
    x: f64,
    y: f64,
    z: f64
}

impl Vector {
	fn clone(&self) -> Vector {
		Vector {
			x: self.x,
			y: self.y,
			z: self.z
		}
	}
}

// Vector add
impl Add<Vector> for Vector {
	type Output = Vector;
	fn add(self, _rhs: Vector) -> Vector {
		Vector {
			x: self.x + _rhs.x,
			y: self.y + _rhs.y,
			z: self.z + _rhs.z
		}
	}
}

// Vector scaling
impl Mul<f64> for Vector {
	type Output = Vector;
	fn mul(self, _rhs: f64) -> Vector {
		Vector {
			x: self.x * _rhs,
			y: self.y * _rhs,
			z: self.z * _rhs
		}
	}
}

// Vector dot product
impl Rem<Vector> for Vector{
	type Output = f64;
	fn rem(self, _rhs: Vector) -> f64 {
		(self.x * _rhs.x) +
		(self.y * _rhs.y) +
		(self.z * _rhs.z)
	}
}

// Cross-product
impl BitXor<Vector> for Vector {
	type Output = Vector;
	fn bitxor(self, _rhs: Vector) -> Vector {
		Vector {
			x: self.y*_rhs.z - self.z*_rhs.y,
			y: self.z*_rhs.x - self.x*_rhs.z,
			z: self.x*_rhs.y - self.y*_rhs.x
		}
	}
}

// Used for vector normalization
impl Not for Vector {
	type Output = Vector;
	fn not(self) -> Vector {
		let dot_product: f64 = self.clone() % self.clone();
		let inv_sqrt: f64 = 1.0 / dot_product.sqrt();
		self * inv_sqrt
	}
}

fn rand_float() -> f64 {
	rand::random::<f64>()
}

// The intersection test for line [o, v]
//     Return 2 if a hit was found (and also return distance and bouncing ray)
//     Return 0 if no hit was found but ray goes upward
//     Return 1 if no hit was found but ray goes downward
fn intersect_test(origin: Vector, direction: Vector) -> (isize, f64, Vector) {
	let spheres: [usize; 9] = [247570,280596,280600,249748,18578,18577,231184,16,16];

	let mut i_flag: isize = 0;
	let mut distance = 1000000000.0;
	let mut vec_toreturn = Vector {
		x: 0.0,
		y: 0.0,
		z: 1.0
	};
	let ray_vert = -origin.z / direction.z;

	if 0.01 < ray_vert {
		distance = ray_vert;
		i_flag = 1;
	}

	for k in 0..19 {
		for j in 0..9 {
			if (spheres[j] & 1usize<<k) > 0 {
				// There is a sphere, but does the ray hit it?
				let p: Vector = origin.clone() + Vector {
					x: -k as f64,
					y: 0.0,
					z: (-(j as isize)-4) as f64
				};
				let b: f64 = p.clone() % direction.clone();
				let c: f64 = p.clone() % p.clone() - 1.0;
				let q: f64 = b*b - c;

				// Does the ray hit the sphere?
				if q > 0.0 {
					// It does, compute the distance camera-sphere
					let s: f64 = -b - q.sqrt();
					if s<distance && s>0.01 {
						// So far this is the minimum distance, save it. And
						// also compute the bouncing ray vector into 'n'
						distance = s;
						vec_toreturn = !(p + direction.clone()*distance.clone());
						i_flag = 2;
					}
				}
			}
		}
	}
	// if i_flag == 1 {
	// 	println!("flag = {:?}", i_flag );
	// }
	(i_flag, distance, vec_toreturn)
}

// Sample the world and return the pixel color for a ray passing by point o
// (Origin) and d (Direction)
fn sample(origin: Vector, direction: Vector) -> Vector {
	let (flag, distance, bounce) = intersect_test(origin.clone(), direction.clone());
	if flag == 0 {
		// No sphere found and the ray goes upward (to the horizon). So,
		// generate a sky color.
		let colorv = Vector {
			x: 0.7,
			y: 0.6,
			z: 1.0
		};
		return colorv * (1.0 - direction.clone().z).powi(4);
	}

	// A sphere was maybe hit.

	// h = intersection coordinate
	let h: Vector = origin + direction.clone()*distance.clone();
	// l = direction to light (with random delta for soft-shadows)
	let l = !(Vector {
		x: 9.0 + rand_float(),
		y: 9.0 + rand_float(),
		z: 16.0
	} + h.clone()*-1.0);
	// r = the half-vector
	let r: Vector = direction.clone() + bounce.clone()*(bounce.clone() % direction.clone() * -2.0);

	// Calculated the lambertian factor
	let mut b: f64 = l.clone() % bounce.clone();

	// Calculate illumination factor (lambertian coefficient > 0 or in shadow)?
	let (shadow_flag, _, _) = intersect_test(h.clone(), l.clone());
	if b < 0.0 || shadow_flag != 0 {
		b = 0.0;
	}

	// Calculate the color 'p' with diffuse and specular component
	let largeb: f64 = if b > 0.0 { 1.0 } else { 0.0 };
	let p: f64 = (l.clone() % r.clone() * largeb).powi(24).powi(2).powi(2);

	if flag == 1 {
		// println!("No sphere was hit.");
		// No sphere was hit and the ray was going downward. Generate a floor
		// color
		let floor_color = h.clone() * 0.2;
		let tmp_h = floor_color.clone();
		let to_return =
			if ((tmp_h.x.ceil() + tmp_h.y.ceil()) as isize) & 1 == 1 {
				Vector {x: 3.0, y: 1.0, z: 1.0}
			} else {
				Vector {x: 3.0, y: 3.0, z: 3.0}
			};
		return to_return * (b*0.2 + 0.1);
	}

	// Only get to here if m == 2
	// A sphere was hit. Cast a ray bouncing from the sphere surface.
	Vector {x:p, y:p, z:p} + sample(h.clone(), r.clone()) * 0.5 // Attenuate color by 50% since it's bouncing (* .5)
}


fn main() {
	let mut image: Vec<u8> = Vec::new();
	// Write the PPM header
	print!("P6 512 512 255 ");

	// Camera direction
	let g = !Vector {x: -6.0, y: -16.0, z: 0.0};
	// Camera up vector ... seams Z is pointing up :/ WTF !
	let a = !(Vector {x: 0.0, y: 0.0, z: 1.0} ^ g.clone()) * 0.002;
	// The right vector, obtained via traditional cross-product
	let b = !(g.clone()^a.clone()) * 0.002;
	// WTF? See https://news.ycombinator.com/item?id=6425965 for more.
	// The mystery `c` point in the `main()` function is the offset from the
	// eye point (ignoring the lens perturbation `t`) to the corner of the
	// focal plane. Note below that we're nominally tracing rays for pixel
	// (x,y) in the direction of ax+by+c. At the image midpoint, this should
	// be `g`.
	let c = (a.clone() + b.clone()) * -256.0 + g;

	for __y in 0..512 {
		let y = 512 - __y;
		for __x in 0..512 {
			let x = 512 - __x;
			// Reuse the vector class to store not XYZ but RGB pixel color
			// Default pixel color should be almost pitch black.
			let mut p = Vector {x:13.0, y:13.0, z:13.0};

			// Cast 64 rays per pixel (for blur (stochastic sampling) and soft-shadows).
			for _ in 0..64{
				// The delta to apply to the origin of the view (for depth of view blur)
				// A little bit of delta up/down and left/right
				let t: Vector = a.clone()*(rand_float() - 0.5)*99.0 + b.clone()*(rand_float() - 0.5)*99.0;

				// Set the camera focal point v(17, 16, 8) and cast the ray.
				// Accumulate the color returned in the p variable.
				p = sample(Vector {x:17.0, y:16.0, z:8.0} + t.clone(), // Ray origin
					!(t.clone()*-1.0 + (a.clone()*(rand_float()+x as f64) +
						b.clone()*(y as f64 +rand_float())+c.clone())*16.0) // Ray direction with random deltas
																// for stochastic sampling
					)*3.5 + p.clone();// +p for color accumulation
			}
			// print!()
			image.push(p.x as u8);
			image.push(p.y as u8);
			image.push(p.z as u8);
		}
	}
	let mut stdout = io::stdout();
	stdout.write(image.as_ref()).unwrap();
}
