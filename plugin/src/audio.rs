use iterpipes::{Pipe, ResetablePipe};
use biquad::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NoteMessage {
    NoteOn(f64),
    NoteOff
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct OscMessage {
    phase: f64,
    muted: bool
}

pub struct NoteController {
    sample_rate: f64,
    phase: f64,
    current_freq_hz: f64,
    muted: bool
}

impl NoteController {
    pub fn new(sample_rate: f64) -> Self {
        NoteController {
            sample_rate,
            phase: 0.0,
            current_freq_hz: 0.0,
            muted: true
        }
    }
}

impl Pipe for NoteController {
    type InputItem = Option<NoteMessage>;
    type OutputItem = OscMessage;

    fn next(&mut self, control: Option<NoteMessage>) -> Self::OutputItem {
        if let Some(message) = control {
            match message {
                NoteMessage::NoteOn(freq) =>  {
                    self.current_freq_hz = freq;
                    self.muted = false;
                },
                NoteMessage::NoteOff => self.muted = true
            }
        }

        let phase_step = self.current_freq_hz / self.sample_rate;

        self.phase = if self.muted && self.phase < 1.41 * phase_step {
            0.0
        } else {
            self.phase + self.current_freq_hz / self.sample_rate
        };
        
        if self.phase > 1.0 {
            self.phase -= 1.0;
        }



        OscMessage {
            phase: self.phase,
            muted: self.phase == 0.0 && self.muted
        }
        
    }
}

impl ResetablePipe for NoteController {
    fn reset(&mut self) {
        self.phase = 0.0;
        self.current_freq_hz = 0.0;
    }
}

pub struct Sine;
impl Pipe for Sine {
    type InputItem = OscMessage;
    type OutputItem = f32;

    fn next(&mut self, osc_msg: Self::InputItem) -> Self::OutputItem {
        use std::f64::consts::TAU;
        if !osc_msg.muted {
            f64::sin(TAU * osc_msg.phase) as f32 * 0.5
        } else {
            0.0
        }
    }
}

pub struct Square;
impl Pipe for Square {
    type InputItem = OscMessage;
    type OutputItem = f32;

    fn next(&mut self, osc_msg: Self::InputItem) -> Self::OutputItem {
        if !osc_msg.muted {
            if osc_msg.phase >= 0.5 {
                1.0
            } else {
                0.0
            }
        } else {
            0.0
        }
    } 
}

pub struct Triangle;
impl Pipe for Triangle {
    type InputItem = OscMessage;
    type OutputItem = f32;

    fn next(&mut self, osc_msg: Self::InputItem) -> Self::OutputItem {
        if !osc_msg.muted {
            2.0 * (-(osc_msg.phase - 0.5).abs() + 0.5) as f32
        } else {
            0.0
        }
    } 
}

pub struct Sawtooth;
impl Pipe for Sawtooth {
    type InputItem = OscMessage;
    type OutputItem = f32;

    fn next(&mut self, osc_msg: Self::InputItem) -> Self::OutputItem {
        if !osc_msg.muted {
            osc_msg.phase as f32
        } else {
            0.0
        }
    } 
}

pub enum Oscillator {
    Sine(Sine),
    Square(Square),
    Triangle(Triangle),
    Sawtooth(Sawtooth),
    Null
}

impl Oscillator {
    pub fn from_control(control: f32) -> Self {
        let control = control as i32;
        match control {
            0 => Self::Sine(Sine),
            1 => Self::Square(Square),
            2 => Self::Triangle(Triangle),
            3 => Self::Sawtooth(Sawtooth),
            _ => Self::Null
        }
    }
}

impl Pipe for Oscillator {
    type InputItem = OscMessage;
    type OutputItem = f32;

    fn next(&mut self, osc_msg: Self::InputItem) -> Self::OutputItem {
        match self {
            Self::Sine(s) => s.next(osc_msg),
            Self::Square(s) => s.next(osc_msg),
            Self::Triangle(s) => s.next(osc_msg),
            Self::Sawtooth(s) => s.next(osc_msg),
            Self::Null => 0.0
        }
    } 
}


// pub struct BiQuad(DirectForm1<f64>);

// impl BiQuad {
//     pub fn make_lpf(fs: f64, fc: f64, q: f64) -> Self {
//         let coeff = Coefficients::<f64>::from_params(Type::LowPass, fs.hz(), fc.hz(), Q_BUTTERWORTH_F64).unwrap();
//         let mut filter = DirectForm1::<f64>::new(coeff);

//         BiQuad(filter)
//     }
// }

// impl Pipe for BiQuad {
//     type InputItem = f32;
//     type OutputItem = f32;

//     fn next(&mut self, x: Self::InputItem) -> Self::OutputItem {
//         self.0.run(x as f64) as f32
//     } 
// }

#[derive(Debug, Clone)]
pub struct BiQuad {
    b0a0: f32,
    b1a0: f32,
    b2a0: f32,
    a1a0: f32,
    a2a0: f32,
    xd: [f32; 2],
    yd: [f32; 2]
}

impl BiQuad {

    pub fn make_lpf(fs: f64, fc: f64, q: f64) -> Self {
        use std::f64::consts::TAU;
        let w0 = TAU * fc/fs;
        let (w0_sin, w0_cos) = w0.sin_cos();
    
        let alpha = w0_sin / (2.0 * q);
    
        let b0 = (1.0 - w0_cos) * 0.5;
        let b1 = 1.0 - w0_cos;
        let b2 = (1.0 - w0_cos) * 0.5;
    
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * w0_cos;
        let a2 = 1.0 - alpha;
    
        BiQuad {
            b0a0: (b0/a0) as f32,
            b1a0: (b1/a0) as f32,
            b2a0: (b2/a0) as f32,
            a1a0: (a1/a0) as f32,
            a2a0: (a2/a0) as f32,
            xd: [0.0 ; 2],
            yd: [0.0 ; 2]
        }
    }

    // pub fn update_lpf(self, fs: f64, fc: f64, q: f64) -> Self {
    //     let mut temp = BiQuad::make_lpf(fs, fc, q);
    
    //     temp.xd = self.xd;
    //     temp.yd = self.yd;

    //     temp
    // }
}

impl Pipe for BiQuad {
    type InputItem = f32;
    type OutputItem = f32;

    fn next(&mut self, x: Self::InputItem) -> Self::OutputItem {
        let y = (self.b0a0 * x) + (self.b1a0 * self.xd[0]) + (self.b2a0 * self.xd[1])
                        - (self.a1a0 * self.yd[0]) - (self.a2a0 * self.yd[1]);

        self.xd[1] = self.xd[0];
        self.xd[0] = x;

        self.yd[1] = self.yd[0];
        self.yd[0] = y;

        y
    } 
}