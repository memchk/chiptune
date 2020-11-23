use lv2::prelude::*;
use wmidi::MidiMessage;
use iterpipes::{LazyMut, Pipe};

// macro_rules! skip_none {
//     ($res:expr) => {
//         match $res {
//             Some(val) => val,
//             None => {
//                 continue;
//             }
//         }
//     };
// }

mod audio;
use audio::{NoteMessage, Oscillator, BiQuad};

#[derive(PortCollection)]
struct Ports {
    control: InputPort<AtomPort>,
    output: OutputPort<Audio>,
    waveform: InputPort<Control>,
    lpf_cutoff: InputPort<Control>
}

#[derive(FeatureCollection)]
pub struct Features<'a> {
    map: LV2Map<'a>,
}

#[derive(URIDCollection)]
pub struct URIDs {
    atom: AtomURIDCollection,
    midi: MidiURIDCollection,
    unit: UnitURIDCollection,
}

#[uri("https://carson.page/lv2/carsonsynth")]
struct CarsonSynth {
    urids: URIDs,
    controller: audio::NoteController,
    sample_rate: f64,
    lpf: (f64, BiQuad)
}

impl Plugin for CarsonSynth
{
    type Ports = Ports;

    type InitFeatures = Features<'static>;
    type AudioFeatures = ();

    fn new(plugin_info: &PluginInfo, features: &mut Features<'static>) -> Option<Self> {

        let sample_rate = plugin_info.sample_rate();
        Some(Self {
            urids: features.map.populate_collection()?,
            sample_rate,
            controller: audio::NoteController::new(sample_rate),
            lpf:(sample_rate, BiQuad::make_lpf(sample_rate, 1000.0, 0.707))
        })
    }

    fn run(&mut self, ports: &mut Ports, _: &mut (), _: u32) {
        
        let midi_urid = self.urids.midi.wmidi.clone();
        
        let input_sequence = ports
            .control
            .read(self.urids.atom.sequence, self.urids.unit.beat)
            .unwrap();

        let mut message_sequence = input_sequence
            .filter_map(|(_, atom)| atom.read(midi_urid, ()))
            .scan((), |(), x|
                match x {
                    MidiMessage::NoteOn(_, note, _) => Some(NoteMessage::NoteOn(note.to_freq_f64())),
                    MidiMessage::NoteOff(_, _, _) => Some(NoteMessage::NoteOff),
                    _ => None
                }
            );
        
        // Check if LPF changed, if so update
        if *ports.lpf_cutoff as f64 != self.lpf.0 {
            self.lpf.0 = *ports.lpf_cutoff as f64;
            self.lpf.1 = BiQuad::make_lpf(self.sample_rate, self.lpf.0, 0.707);
        }

        let mut pipeline = LazyMut::new(|_ :()| message_sequence.next()).compose()
            >> (&mut self.controller)
            >> Oscillator::from_control(*ports.waveform)
            >> &mut self.lpf.1;

        for frame in ports.output.iter_mut() {
            *frame = pipeline.next(());
        }
    }
}

lv2_descriptors!(CarsonSynth);