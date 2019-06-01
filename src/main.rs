use glium::{glutin, Surface};
use rand::distributions::*;
use rand::Rng;

#[derive(Copy, Clone)]
struct Particle {
    pub mass: f32,
    pub charge: f32,
    pub velocity: [f32; 2],
    pub position: [f32; 2],
}

fn magnitude(value: [f32; 2]) -> f32 {
    ((value[0] * value[0]) + (value[1] * value[1])).sqrt()
}

fn normalize(value: [f32; 2]) -> [f32; 2] {
    let magnitude = magnitude(value);
    return [value[0] / magnitude, value[1] / magnitude];
}

fn subtract(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn add(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] + b[0], a[1] + b[1]]
}

fn scalar_mul(a: [f32; 2], b: f32) -> [f32; 2] {
    [a[0] * b, a[1] * b]
}

impl Particle {
    fn simulate_motion_step(&mut self) {
        self.position = add(self.position, self.velocity);
    }

    fn simulate_force(&mut self, other: &Particle) {
        let line_segment = subtract(self.position, other.position);
        let r = magnitude(line_segment);
        let cpd = (self.charge * other.charge) / (r * r);
        let unit_direction = normalize(line_segment);
        let f = scalar_mul(unit_direction, cpd / self.mass);
        self.velocity = add(self.velocity, f);
    }
}

fn particle_sim(particles: &mut Vec<Particle>) {
    for x in 0..particles.len() {
        let current_particle = particles[x].clone();
        for y in (0..x).chain(x + 1..particles.len()) {
            particles[y].simulate_force(&current_particle);
        }
        particles[x].simulate_motion_step();
    }
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

glium::implement_vertex!(Vertex, position, color);

fn main() {
    let mut particles = Vec::new();
    let position_dist = Uniform::new(-1.0, 1.0);
    let charge_dist = Normal::new(0.0, 0.03);

    let mut rng = rand::thread_rng();
    for _ in 0..1000 {
        particles.push(Particle {
            mass: 100000.0,
            charge: charge_dist.sample(&mut rng) as f32,
            velocity: [0.0, 0.0],
            position: [
                position_dist.sample(&mut rng) as f32,
                position_dist.sample(&mut rng) as f32,
            ],
        })
    }

    let mut events_loop = glutin::EventsLoop::new();
    let wb = glutin::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &events_loop).unwrap();

    let vertex_buffer: glium::VertexBuffer<Vertex> =
        glium::VertexBuffer::empty_dynamic(&display, particles.len()).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::Points);
    let mut params = glium::DrawParameters::default();
    params.point_size = Some(8.0);

    let vertex_shader_src = r#"
        #version 140

        in vec2 position;
        in vec3 color;
        out vec3 vcolor;

        void main() {
            gl_Position = vec4(position, 0.0, 1.0);
            vcolor = color;
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        out vec4 color;
        in vec3 vcolor;

        void main() {
            color = vec4(vcolor, 1.0);
        }
    "#;

    let program =
        glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None)
        .unwrap();

    let mut closed = false;
    while !closed {
        particle_sim(&mut particles);

        let mut display_points = Vec::with_capacity(particles.len());
        for particle in &particles {
            display_points.push(Vertex {
                position: particle.position,
                color: if particle.charge > 0.0 {
                    [1.0, 0.2, 0.2]
                } else {
                    [0.2, 0.4, 1.0]
                },
            });
        }
        vertex_buffer.write(&display_points);

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target
            .draw(
                &vertex_buffer,
                &indices,
                &program,
                &glium::uniforms::EmptyUniforms,
                &params,
                )
            .unwrap();
        target.finish().unwrap();

        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => closed = true,
                _ => (),
            },
            _ => (),
        });
    }
}
