use std::{f32::consts::FRAC_PI_2, time::Duration};

use cgmath::{InnerSpace, Matrix4, Point3, Rad, Vector3, perspective};

pub const PROJECTION_CORRECTION_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0,  0.0,  0.0,  0.0,
    0.0, -1.0,  0.0,  0.0,
    0.0,  0.0,  0.5,  0.0,
    0.0,  0.0,  0.5,  1.0,
);

pub struct Camera {
    pub position: Point3<f32>,
    pub pitch: Rad<f32>,
    pub yaw: Rad<f32>,
    // pub sensitivity: f32,
    // pub movement_speed: f32,
}

pub struct Projection {
    aspect: f32,
    fov_y: Rad<f32>,
    z_near: f32,
    z_far: f32,
}

pub enum CameraMove {
    Up,
    Down,
    Left,
    Right
}

pub struct CameraController {
    pub amount_left: f32,
    pub amount_right: f32,
    pub amount_forward: f32,
    pub amount_backward: f32,
    pub amount_up: f32,
    pub amount_down: f32,
    pub rotate_horizontal: f32,
    pub rotate_vertical: f32,
    pub scroll: f32,
    pub speed: f32,
    pub sensitivity: f32,
}


impl Camera {
    pub fn new(position: Point3<f32>, pitch: Rad<f32>, yaw: Rad<f32>) -> Self {
        Self { position, pitch, yaw }
    }

    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Matrix4::look_to_rh(
            self.position,
            Vector3::new(
                cos_pitch * cos_yaw,
                sin_pitch,
                cos_pitch * sin_yaw
            ).normalize(),
            Vector3::unit_y(),
        )
    }

    // pub fn rotate(&mut self, mouse_x_diff: f32, mouse_y_diff: f32, delta_time: f32) {
    //     self.yaw += Rad(mouse_x_diff) * self.sensitivity * delta_time;
    //     self.pitch += Rad(mouse_y_diff) * self.sensitivity * delta_time;
    // }

    // pub fn move_horizontal(&mut self, movement: CameraMove, delta_time: f32) {
    //     let (yaw_sin, yaw_cos) = self.yaw.0.sin_cos();
    //     let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
    //     let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
    //     self.position += forward * (self.amount_forward - self.amount_backward) * self.movement_speed * delta_time;
    //     self.position += right * (self.amount_right - self.amount_left) * self.movement_speed * delta_time;
    // }

    // pub fn move_vertical(&mut self, amount_up: f32, delta_time: f32) {
    //     self.position.y += amount_up * self.movement_speed * delta_time;
    // }
}

impl Projection {
    pub fn new(width: u32, height: u32, fov_y: Rad<f32>, z_near: f32, z_far: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fov_y,
            z_near,
            z_far,
        }
    }

    pub fn get_perspective_projection_matrix(&self) -> Matrix4<f32> {
        perspective(self.fov_y, self.aspect, self.z_near, self.z_far) * PROJECTION_CORRECTION_MATRIX
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }
}

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let (pitch_sin, pitch_cos) = camera.pitch.0.sin_cos();
        // let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let forward = Vector3::new(
            pitch_cos * yaw_cos,
            pitch_sin,
            pitch_cos * yaw_sin,
        ).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.position += right * -(self.amount_right - self.amount_left) * self.speed * dt;


        camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        camera.yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
        camera.pitch += Rad(self.rotate_vertical) * self.sensitivity * dt;

        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;


    }
}