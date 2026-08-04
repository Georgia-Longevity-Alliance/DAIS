//! Swarm Coordination — distributed task allocation for robot fleets.
//!
//! Inspired by Kilobots (Harvard), ARGoS simulator, and Buzz language.
//! Uses lightweight consensus for task distribution, collision avoidance,
//! and emergent behavior without central control.
//!
//! Key algorithms:
//! - Task Auction: robots bid on tasks based on capability match
//! - Virtual Stigmergy: shared state via tuple spaces
//! - Flocking: Boids-inspired collision avoidance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A task to be executed by the swarm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmTask {
    pub task_id: String,
    pub task_type: TaskType,
    pub required_capability: String,
    pub priority: u8,           // 0-255, higher = more urgent
    pub deadline_secs: Option<f64>,
    pub assigned_to: Option<String>,
    pub status: TaskStatus,
    pub location: Option<SwarmPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskType {
    Explore,
    Acquire,
    Transport,
    Monitor,
    Charge,
    Assist,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Auctioning,
    Assigned,
    InProgress,
    Completed,
    Failed,
}

/// Position in 2D/3D space
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SwarmPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// A robot in the swarm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmRobot {
    pub robot_id: String,
    pub position: SwarmPosition,
    pub velocity: (f64, f64, f64),
    pub capabilities: Vec<String>,
    pub battery: f64,          // 0.0 - 1.0
    pub current_task: Option<String>,
    pub role: SwarmRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SwarmRole {
    Worker,
    Scout,
    Charger,
    Coordinator,
}

/// Swarm Coordinator — manages distributed task allocation
pub struct SwarmCoordinator {
    pub robots: HashMap<String, SwarmRobot>,
    pub tasks: HashMap<String, SwarmTask>,
    pub completed_tasks: u64,
    pub collisions_avoided: u64,
}

impl SwarmCoordinator {
    pub fn new() -> Self {
        Self { robots: HashMap::new(), tasks: HashMap::new(), completed_tasks: 0, collisions_avoided: 0 }
    }

    /// Register a robot in the swarm
    pub fn register_robot(&mut self, robot: SwarmRobot) {
        self.robots.insert(robot.robot_id.clone(), robot);
    }

    /// Submit a new task
    pub fn submit_task(&mut self, task: SwarmTask) -> String {
        let id = task.task_id.clone();
        self.tasks.insert(id.clone(), task);
        id
    }

    /// Run one round of task auction
    pub fn auction_round(&mut self) -> Vec<(String, String)> {
        let mut assignments = Vec::new();
        let unassigned: Vec<SwarmTask> = self.tasks.values()
            .filter(|t| t.status == TaskStatus::Pending || t.status == TaskStatus::Auctioning)
            .cloned()
            .collect();

        for task in &unassigned {
            let mut best_bid = 0.0f64;
            let mut best_robot: Option<String> = None;

            for robot in self.robots.values() {
                if robot.current_task.is_some() { continue; }
                if !robot.capabilities.iter().any(|c| c.contains(&task.required_capability)) { continue; }
                
                let bid = self.calculate_bid(robot, task);
                if bid > best_bid {
                    best_bid = bid;
                    best_robot = Some(robot.robot_id.clone());
                }
            }

            if let Some(ref robot_id) = best_robot {
                if let Some(task) = self.tasks.get_mut(&task.task_id) {
                    task.status = TaskStatus::Assigned;
                    task.assigned_to = Some(robot_id.clone());
                }
                if let Some(robot) = self.robots.get_mut(robot_id) {
                    robot.current_task = Some(task.task_id.clone());
                }
                assignments.push((robot_id.clone(), task.task_id.clone()));
            }
        }
        assignments
    }

    /// Calculate bid — higher is better
    fn calculate_bid(&self, robot: &SwarmRobot, task: &SwarmTask) -> f64 {
        let capability_match = if robot.capabilities.iter().any(|c| c.contains(&task.required_capability)) { 1.0 } else { 0.0 };
        let battery_factor = robot.battery;
        let priority_factor = task.priority as f64 / 255.0;
        let deadline_factor = if let Some(deadline) = task.deadline_secs {
            if deadline > 0.0 { 1.0 / deadline } else { 1.0 }
        } else { 0.5 };

        capability_match * 0.4 + battery_factor * 0.3 + priority_factor * 0.2 + deadline_factor * 0.1
    }

    /// Boids flocking — collision avoidance
    pub fn flocking_step(&mut self, separation_distance: f64) {
        let positions: Vec<(String, SwarmPosition)> = self.robots.iter()
            .map(|(id, r)| (id.clone(), r.position))
            .collect();

        for (id, pos) in &positions {
            for (other_id, other_pos) in &positions {
                if id == other_id { continue; }
                
                let dx = pos.x - other_pos.x;
                let dy = pos.y - other_pos.y;
                let dist = (dx * dx + dy * dy).sqrt();
                
                if dist < separation_distance && dist > 0.0 {
                    if let Some(robot) = self.robots.get_mut(id) {
                        robot.velocity.0 += dx / dist * 0.1;
                        robot.velocity.1 += dy / dist * 0.1;
                    }
                    self.collisions_avoided += 1;
                }
            }
        }

        // Apply velocities
        for robot in self.robots.values_mut() {
            robot.position.x += robot.velocity.0;
            robot.position.y += robot.velocity.1;
            robot.velocity.0 *= 0.9;
            robot.velocity.1 *= 0.9;
        }
    }

    /// Complete a task
    pub fn complete_task(&mut self, task_id: &str) -> bool {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.status = TaskStatus::Completed;
            if let Some(ref robot_id) = task.assigned_to.clone() {
                if let Some(robot) = self.robots.get_mut(robot_id) {
                    robot.current_task = None;
                }
            }
            self.completed_tasks += 1;
            return true;
        }
        false
    }

    /// Swarm status
    pub fn status(&self) -> SwarmStatus {
        SwarmStatus {
            num_robots: self.robots.len() as u64,
            num_tasks: self.tasks.len() as u64,
            tasks_pending: self.tasks.values().filter(|t| t.status == TaskStatus::Pending).count() as u64,
            tasks_in_progress: self.tasks.values().filter(|t| t.status == TaskStatus::InProgress).count() as u64,
            tasks_completed: self.completed_tasks,
            collisions_avoided: self.collisions_avoided,
            avg_battery: self.robots.values().map(|r| r.battery).sum::<f64>() / self.robots.len().max(1) as f64,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwarmStatus {
    pub num_robots: u64,
    pub num_tasks: u64,
    pub tasks_pending: u64,
    pub tasks_in_progress: u64,
    pub tasks_completed: u64,
    pub collisions_avoided: u64,
    pub avg_battery: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_auction() {
        let mut swarm = SwarmCoordinator::new();
        
        swarm.register_robot(SwarmRobot {
            robot_id: "r1".into(), position: SwarmPosition { x: 0.0, y: 0.0, z: 0.0 },
            velocity: (0.0, 0.0, 0.0), capabilities: vec!["z_stack".into()],
            battery: 0.9, current_task: None, role: SwarmRole::Worker,
        });
        swarm.register_robot(SwarmRobot {
            robot_id: "r2".into(), position: SwarmPosition { x: 1.0, y: 0.0, z: 0.0 },
            velocity: (0.0, 0.0, 0.0), capabilities: vec!["transport".into()],
            battery: 0.5, current_task: None, role: SwarmRole::Worker,
        });

        swarm.submit_task(SwarmTask {
            task_id: "t1".into(), task_type: TaskType::Acquire,
            required_capability: "z_stack".into(), priority: 200,
            deadline_secs: None, assigned_to: None, status: TaskStatus::Pending,
            location: None,
        });

        let assignments = swarm.auction_round();
        assert_eq!(assignments.len(), 1);
        assert_eq!(assignments[0].0, "r1"); // r1 wins because it has the capability
    }

    #[test]
    fn test_flocking_collision_avoidance() {
        let mut swarm = SwarmCoordinator::new();
        // Two robots dangerously close
        swarm.register_robot(SwarmRobot {
            robot_id: "r1".into(), position: SwarmPosition { x: 0.0, y: 0.0, z: 0.0 },
            velocity: (0.0, 0.0, 0.0), capabilities: vec![], battery: 1.0,
            current_task: None, role: SwarmRole::Worker,
        });
        swarm.register_robot(SwarmRobot {
            robot_id: "r2".into(), position: SwarmPosition { x: 0.1, y: 0.0, z: 0.0 },
            velocity: (0.0, 0.0, 0.0), capabilities: vec![], battery: 1.0,
            current_task: None, role: SwarmRole::Worker,
        });
        
        swarm.flocking_step(1.0);
        assert!(swarm.collisions_avoided > 0);
    }

    #[test]
    fn test_task_completion() {
        let mut swarm = SwarmCoordinator::new();
        swarm.register_robot(SwarmRobot {
            robot_id: "r1".into(), position: SwarmPosition { x: 0.0, y: 0.0, z: 0.0 },
            velocity: (0.0, 0.0, 0.0), capabilities: vec!["scan".into()],
            battery: 1.0, current_task: None, role: SwarmRole::Worker,
        });
        swarm.submit_task(SwarmTask {
            task_id: "t1".into(), task_type: TaskType::Monitor,
            required_capability: "scan".into(), priority: 100,
            deadline_secs: None, assigned_to: None, status: TaskStatus::Pending,
            location: None,
        });
        
        swarm.auction_round();
        assert!(swarm.complete_task("t1"));
        assert_eq!(swarm.completed_tasks, 1);
    }
}
