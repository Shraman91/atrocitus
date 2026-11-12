use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Copy, Clone, Eq, PartialEq)]
struct Node {
    idx: usize,
    f_cost: u32,
    g_cost: u32,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_cost.cmp(&self.f_cost)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct NavGrid {
    pub cell_size: f32,
    pub cols: usize,
    pub rows: usize,
    pub walkable: Vec<bool>,
}

impl NavGrid {
    pub fn new(width_px: f32, height_px: f32, cell_size: f32) -> Self {
        let cols = (width_px / cell_size).ceil() as usize;
        let rows = (height_px / cell_size).ceil() as usize;
        Self {
            cell_size,
            cols,
            rows,
            walkable: vec![true; cols * rows],
        }
    }

    #[inline]
    pub fn world_to_cell(&self, wx: f32, wy: f32) -> (usize, usize) {
        let c = ((wx / self.cell_size) as usize).min(self.cols.saturating_sub(1));
        let r = ((wy / self.cell_size) as usize).min(self.rows.saturating_sub(1));
        (c, r)
    }

    #[inline]
    pub fn cell_to_world(&self, c: usize, r: usize) -> (f32, f32) {
        (
            (c as f32 + 0.5) * self.cell_size,
            (r as f32 + 0.5) * self.cell_size,
        )
    }

    #[inline]
    pub fn is_walkable(&self, c: usize, r: usize) -> bool {
        if c < self.cols && r < self.rows {
            self.walkable[r * self.cols + c]
        } else {
            false
        }
    }

    pub fn set_blocked_rect(&mut self, min_x: f32, min_y: f32, max_x: f32, max_y: f32) {
        let (c0, r0) = self.world_to_cell(min_x, min_y);
        let (c1, r1) = self.world_to_cell(max_x, max_y);

        for r in r0..=r1 {
            for c in c0..=c1 {
                if c < self.cols && r < self.rows {
                    self.walkable[r * self.cols + c] = false;
                }
            }
        }
    }

    /// Finds a smooth path of world waypoints from `start` to `target`.
    pub fn find_path(&self, start: (f32, f32), target: (f32, f32)) -> Option<Vec<(f32, f32)>> {
        let (start_c, start_r) = self.world_to_cell(start.0, start.1);
        let (goal_c, goal_r) = self.world_to_cell(target.0, target.1);

        if !self.is_walkable(goal_c, goal_r) {
            // Find closest walkable cell to goal
            let mut best_cell = None;
            let mut min_d = f32::MAX;
            for dr in -2..=2 {
                for dc in -2..=2 {
                    let nc = goal_c as i32 + dc;
                    let nr = goal_r as i32 + dr;
                    if nc >= 0 && nr >= 0 && self.is_walkable(nc as usize, nr as usize) {
                        let d = (dc * dc + dr * dr) as f32;
                        if d < min_d {
                            min_d = d;
                            best_cell = Some((nc as usize, nr as usize));
                        }
                    }
                }
            }
            if let Some(closest) = best_cell {
                return self.run_astar(start_c, start_r, closest.0, closest.1, target);
            }
            return None;
        }

        self.run_astar(start_c, start_r, goal_c, goal_r, target)
    }

    fn run_astar(
        &self,
        start_c: usize,
        start_r: usize,
        goal_c: usize,
        goal_r: usize,
        final_exact: (f32, f32),
    ) -> Option<Vec<(f32, f32)>> {
        let total_cells = self.cols * self.rows;
        let mut dist = vec![u32::MAX; total_cells];
        let mut parent = vec![usize::MAX; total_cells];
        let mut heap = BinaryHeap::with_capacity(128);

        let start_idx = start_r * self.cols + start_c;
        let goal_idx = goal_r * self.cols + goal_c;

        dist[start_idx] = 0;
        let start_h = (((goal_c as i32 - start_c as i32).abs() + (goal_r as i32 - start_r as i32).abs()) * 10) as u32;
        heap.push(Node {
            idx: start_idx,
            f_cost: start_h,
            g_cost: 0,
        });

        let dirs: [(i32, i32, u32); 8] = [
            (1, 0, 10),
            (-1, 0, 10),
            (0, 1, 10),
            (0, -1, 10),
            (1, 1, 14),
            (1, -1, 14),
            (-1, 1, 14),
            (-1, -1, 14),
        ];

        while let Some(Node { idx, g_cost, .. }) = heap.pop() {
            if idx == goal_idx {
                let mut path = Vec::with_capacity(32);
                let mut curr = goal_idx;
                while curr != start_idx && curr != usize::MAX {
                    let c = curr % self.cols;
                    let r = curr / self.cols;
                    path.push(self.cell_to_world(c, r));
                    curr = parent[curr];
                }
                path.reverse();
                if let Some(last) = path.last_mut() {
                    *last = final_exact;
                }
                return Some(path);
            }

            if g_cost > dist[idx] {
                continue;
            }

            let c = (idx % self.cols) as i32;
            let r = (idx / self.cols) as i32;

            for &(dc, dr, move_cost) in &dirs {
                let nc = c + dc;
                let nr = r + dr;

                if nc >= 0 && nc < self.cols as i32 && nr >= 0 && nr < self.rows as i32 {
                    let u_nc = nc as usize;
                    let u_nr = nr as usize;

                    if self.is_walkable(u_nc, u_nr) {
                        let next_idx = u_nr * self.cols + u_nc;
                        let next_g = dist[idx].saturating_add(move_cost);

                        if next_g < dist[next_idx] {
                            dist[next_idx] = next_g;
                            parent[next_idx] = idx;
                            let h = (((goal_c as i32 - nc).abs() + (goal_r as i32 - nr).abs()) * 10) as u32;
                            heap.push(Node {
                                idx: next_idx,
                                f_cost: next_g + h,
                                g_cost: next_g,
                            });
                        }
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nav_grid_straight_path() {
        let grid = NavGrid::new(100.0, 100.0, 10.0);
        let path = grid.find_path((5.0, 5.0), (85.0, 5.0));
        assert!(path.is_some());
        let pts = path.unwrap();
        assert!(!pts.is_empty());
        assert_eq!(*pts.last().unwrap(), (85.0, 5.0));
    }

    #[test]
    fn test_nav_grid_obstacle_avoidance() {
        let mut grid = NavGrid::new(100.0, 100.0, 10.0);
        // Block intermediate column at c = 4
        grid.set_blocked_rect(40.0, 0.0, 49.0, 50.0);
        let path = grid.find_path((10.0, 20.0), (70.0, 20.0));
        assert!(path.is_some());
        let pts = path.unwrap();
        assert!(!pts.is_empty());
        for pt in &pts {
            let (c, r) = grid.world_to_cell(pt.0, pt.1);
            if r <= 5 {
                assert_ne!(c, 4);
            }
        }
    }
}
