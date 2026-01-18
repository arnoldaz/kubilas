use std::{f32::consts::FRAC_PI_2, time::Duration};
use cgmath::{InnerSpace, Matrix4, Point3, Rad, Vector3, perspective};

pub const PROJECTION_CORRECTION_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0,  0.0,  0.0,  0.0,
    0.0, -1.0,  0.0,  0.0,
    0.0,  0.0,  0.5,  0.0,
    0.0,  0.0,  0.5,  1.0,
);

pub struct Camera {
    position: Point3<f32>,
    pitch: Rad<f32>,
    yaw: Rad<f32>,
}

pub struct Projection {
    aspect: f32,
    fov_y: Rad<f32>,
    z_near: f32,
    z_far: f32,
}

pub struct CameraMovement {
    pub amount_forward: f32,
    pub amount_backward: f32,
    pub amount_left: f32,
    pub amount_right: f32,
    pub amount_up: f32,
    pub amount_down: f32,
    pub rotate_horizontal: f32,
    pub rotate_vertical: f32,
    speed: f32,
    sensitivity: f32,
}

impl Camera {
    pub fn new<P3: Into<Point3<f32>>, P: Into<Rad<f32>>, Y: Into<Rad<f32>>>(position: P3, pitch: P, yaw: Y) -> Self {
        Self { position: position.into(), pitch: pitch.into(), yaw: yaw.into() }
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

    pub fn update(&mut self, movement: &CameraMovement, delta_time: Duration) {
        let delta_time = delta_time.as_secs_f32();

        let (yaw_sin, yaw_cos) = self.yaw.0.sin_cos();
        let (pitch_sin, pitch_cos) = self.pitch.0.sin_cos();

        let forward = Vector3::new(
            pitch_cos * yaw_cos,
            pitch_sin,
            pitch_cos * yaw_sin,
        ).normalize();

        let world_up = Vector3::new(0.0, 1.0, 0.0);

        let right = forward.cross(world_up).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();

        // Forwards/backwards
        self.position += forward * (movement.amount_forward - movement.amount_backward) * movement.speed * delta_time;
        
        // Left/right
        self.position += right * (movement.amount_right - movement.amount_left) * movement.speed * delta_time;

        // Up/down
        self.position.y += (movement.amount_up - movement.amount_down) * movement.speed * delta_time;

        // Rotate
        self.yaw += Rad(movement.rotate_horizontal) * movement.sensitivity;
        self.pitch -= Rad(movement.rotate_vertical) * movement.sensitivity;

        // Prevent camera flips
        let max_pitch = FRAC_PI_2 - 0.01;
        self.pitch.0 = self.pitch.0.clamp(-max_pitch, max_pitch);
    }
}

impl Projection {
    pub fn new<P: Into<Rad<f32>>>(width: u32, height: u32, fov_y: P, z_near: f32, z_far: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fov_y: fov_y.into(),
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

impl CameraMovement {
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
            speed,
            sensitivity,
        }
    }
}
