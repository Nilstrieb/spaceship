use glam::DVec2;

#[derive(Debug, Clone, Copy)]
pub struct Orbit {
    pub semi_major_axis: f64,
    pub eccentricity: f64,
}

pub const G: f64 = 6.6e-11;

fn cartesian_to_polar(pos: DVec2) -> (f64, f64) {
    (pos.length(), f64::atan(pos.y / pos.x))
}

impl Orbit {
    pub fn from_pos_dir(m: f64, pos: DVec2, v: DVec2) -> Orbit {
        let (r, theta) = cartesian_to_polar(pos);
        let (v, psi) = cartesian_to_polar(v);

        // https://phys.libretexts.org/Bookshelves/Astronomy__Cosmology/Celestial_Mechanics_(Tatum)/09%3A_The_Two_Body_Problem_in_Two_Dimensions/9.08%3A_Orbital_Elements_and_Velocity_Vector

        // semi major axis, 9.5.31
        // a = (GMr)/(2GM-v^2r)
        let a = (G * m * r) / ((2.0 * G * m) - (v * v * r));

        // eccentricity, 9.9.3
        // rV sin(psi - theta) = sqrt(GMa(1-e^2))
        let rvsin = r * v * (psi - theta).sin();
        let gma = G * m * a;

        let e = f64::sqrt((-(rvsin * rvsin - gma)) / gma);

        Orbit {
            semi_major_axis: a,
            eccentricity: e,
        }
    }

    pub fn periapsis(&self) -> f64 {
        self.semi_major_axis * (1.0 - self.eccentricity)
    }

    pub fn apoapsis(&self) -> f64 {
        self.semi_major_axis * (1.0 + self.eccentricity)
    }
}

#[cfg(test)]
mod tests {
    use glam::DVec2;

    #[test]
    fn geostationary() {
        let orbit =
            super::Orbit::from_pos_dir(5.972e24, DVec2::new(42000.0, 0.0), DVec2::new(0.0, 3074.0));
        assert!(
            (21000.0 - orbit.semi_major_axis) < 20.0,
            "{} == {}",
            21000.0,
            orbit.semi_major_axis
        );
        // this is sus.
        assert!(
            (1.0 - orbit.eccentricity).abs() < 0.1,
            "{} == {}",
            1.0,
            orbit.eccentricity
        );
    }
}
