use alvinw_chess::pos::BoardPos;

pub struct MoveAnimation {
    pub running: bool,
    animation_start_time: u64,
    duration: u64,
    move_from: Option<BoardPos>,
    move_dest: Option<BoardPos>
}

impl MoveAnimation {
    pub fn new() -> MoveAnimation {
        MoveAnimation {
            running: false,
            animation_start_time: 0,
            duration: 0,
            move_from: None,
            move_dest: None
        }
    }

    pub fn target(&self) -> Option<BoardPos> {
        self.move_dest.clone()
    }

    pub fn set_animation(&mut self, start_time: u64, duration: u64, from: &BoardPos, to: &BoardPos) {
        if !self.running {
            self.running = true;
            self.animation_start_time = start_time;
            self.duration = duration;
            self.move_from = Some(from.clone());
            self.move_dest = Some(to.clone());
        }
    }

    pub fn cancel_current_animation(&mut self) {
        self.running = false;
    }

    pub fn update_animation(&mut self, current_time: u64) -> Option<(f64, f64)> {

        if let Some(dest) = self.move_dest.clone() {
            if !self.running {
                self.running = current_time >= self.animation_start_time && 
                    current_time < self.animation_start_time + self.duration;
            }
    
            if self.running {
                let animation_progress: f64 = ((current_time as f64 - self.animation_start_time as f64) / self.duration as f64).sqrt();

                if animation_progress >= 1.0 {
                    self.running = false;
                    return Some((dest.file() as f64, dest.rank() as f64));
                }
                else {
                    let from = self.move_from.clone().expect("Always exists it there is a destination");
                    let rank = from.rank() as f64 + (dest.rank() as f64 - from.rank() as f64) * animation_progress;
                    let file = from.file() as f64 + (dest.file() as f64 - from.file() as f64) * animation_progress;
                    return Some((file, rank));
                }
            }
        }
        return None;
    }
}