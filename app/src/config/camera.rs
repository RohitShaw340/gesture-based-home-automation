use std::sync::OnceLock;

// TODO: Vec3A or Vec3
use glam::{EulerRot, Quat, Vec3A};
use serde::Deserialize;

use crate::{HasGlamPosition, HasGlamQuat};

#[derive(Deserialize, Debug)]
pub struct CameraProperties {
    pub fov_x: f32,
    pub fov_y: f32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
    pub img_height: u32,
    pub img_width: u32,
    pub intrensic_prams: [[f64; 3]; 3],
    pub rotation_matrix: [[f64; 3]; 3],
    #[serde(skip)]
    quat: OnceLock<Quat>,
    #[serde(skip)]
    dir_vec: OnceLock<Vec3A>,
    #[serde(skip)]
    pos: OnceLock<Vec3A>,
}

impl CameraProperties {
    pub fn test_new() -> Self {
        let sample_intrensic_matrix = [
            [1.4253555975305719e+03, 0., 7.2552788750799868e+02],
            [0., 1.4039605486267199e+03, 4.0030984906993211e+02],
            [0., 0., 1.],
        ];
        let sample_rotation_matrix = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        CameraProperties {
            pos_x: 0.0,
            pos_y: 0.0,
            pos_z: 0.0,
            fov_x: 1.0472, // 60 degrees
            fov_y: 0.58905,
            pitch: 0.0,
            yaw: 0.0,
            roll: 0.0,
            intrensic_prams: &sample_intrensic_matrix,
            rotation_matrix: &sample_rotation_matrix,
            img_height: 720,
            img_width: 1280,
            quat: OnceLock::new(),
            dir_vec: OnceLock::new(),
            pos: OnceLock::new(),
        }
    }

    pub fn direction_vector(&self) -> &Vec3A {
        self.dir_vec
            .get_or_init(|| self.quat().mul_vec3a(crate::math::BASE_FORWARD_VECTOR))
    }

    pub fn forward_vector(&self) -> &Vec3A {
        &crate::math::BASE_FORWARD_VECTOR
    }
}

impl HasGlamPosition for CameraProperties {
    fn pos(&self) -> &Vec3A {
        self.pos
            .get_or_init(|| Vec3A::new(self.pos_x, self.pos_y, self.pos_z))
    }
}

impl HasGlamQuat for CameraProperties {
    fn quat(&self) -> Quat {
        *self
            .quat
            .get_or_init(|| Quat::from_euler(EulerRot::ZYX, self.yaw, self.pitch, self.roll))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dir_vec() {
        let sample_intrensic_matrix = [
            [1.4253555975305719e+03, 0., 7.2552788750799868e+02],
            [0., 1.4039605486267199e+03, 4.0030984906993211e+02],
            [0., 0., 1.],
        ];
        let sample_rotation_matrix = [
            [
                9.9165936444845415e-01,
                8.3969100257135582e-02,
                9.7779829738525781e-02,
            ],
            [
                -8.9473106805596891e-02,
                9.9456050000375096e-01,
                5.3328932024213592e-02,
            ],
            [
                -9.2769973915282689e-02,
                -6.1632799987474653e-02,
                9.9377820961493302e-01,
            ],
        ];
        let camera = CameraProperties {
            fov_x: 1.0,
            fov_y: 1.0,
            pos_x: 69.0,
            pos_y: 69.0,
            pos_z: 69.0,
            pitch: 0.2,
            yaw: 0.69,
            roll: -0.69,
            intrensic_prams: &sample_intrensic_matrix,
            rotation_matrix: &sample_rotation_matrix,
            img_height: 720,
            img_width: 1280,
            quat: OnceLock::new(),
            dir_vec: OnceLock::new(),
            pos: OnceLock::new(),
        };

        assert_eq!(
            Vec3A::new(0.7558725, 0.62384874, -0.19866931),
            *camera.direction_vector()
        )
    }
}
