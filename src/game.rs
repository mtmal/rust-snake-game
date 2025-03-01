use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Represents a point in 2D space
/// Used for both snake body segments and food position
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// Represents the possible directions the snake can move
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Main game state structure
#[derive(Serialize, Deserialize)]
pub struct Game {
    /// Snake body represented as a double-ended queue
    /// First element is the head, last is the tail
    pub snake: VecDeque<Point>,
    /// Current position of the food
    pub food: Point,
    /// Current direction of snake movement
    pub direction: Direction,
    /// Number of food items eaten (score)
    pub score: u32,
    /// Whether the game has ended
    pub game_over: bool,
    /// Game board width
    pub width: i32,
    /// Game board height
    pub height: i32,
}

impl Game {
    /// Creates a new game instance with specified dimensions
    /// Initializes snake at the center of the board
    pub fn new(width: i32, height: i32) -> Self {
        let mut game = Game {
            snake: VecDeque::new(),
            food: Point { x: 0, y: 0 },
            direction: Direction::Right,
            score: 0,
            game_over: false,
            width,
            height,
        };

        // Initialize snake at the center
        game.snake.push_back(Point {
            x: width / 2,
            y: height / 2,
        });
        game.spawn_food();
        game
    }

    /// Updates the game state for one time step
    /// Handles movement, collisions, and food consumption
    pub fn update(&mut self) {
        if self.game_over {
            return;
        }

        let head = self.snake.front().unwrap();
        // Calculate new head position based on current direction
        let new_head = match self.direction {
            Direction::Up => Point {
                x: head.x,
                y: head.y - 1,
            },
            Direction::Down => Point {
                x: head.x,
                y: head.y + 1,
            },
            Direction::Left => Point {
                x: head.x - 1,
                y: head.y,
            },
            Direction::Right => Point {
                x: head.x + 1,
                y: head.y,
            },
        };

        // Check collision with walls
        if new_head.x < 0
            || new_head.x >= self.width
            || new_head.y < 0
            || new_head.y >= self.height
        {
            self.game_over = true;
            return;
        }

        // Check collision with self
        if self.snake.contains(&new_head) {
            self.game_over = true;
            return;
        }

        // Add new head to snake
        self.snake.push_front(new_head);

        // Check if food is eaten
        if new_head.x == self.food.x && new_head.y == self.food.y {
            self.score += 1;
            self.spawn_food();
        } else {
            // Remove tail if food wasn't eaten
            self.snake.pop_back();
        }
    }

    /// Spawns new food at a random position
    /// Ensures food doesn't spawn on snake body
    pub fn spawn_food(&mut self) {
        let mut rng = rand::thread_rng();
        loop {
            let food = Point {
                x: rng.gen_range(0..self.width),
                y: rng.gen_range(0..self.height),
            };
            if !self.snake.contains(&food) {
                self.food = food;
                break;
            }
        }
    }

    /// AI control function that chooses the next move
    /// Uses a simple algorithm to move towards food while avoiding obstacles
    pub fn ai_move(&mut self) {
        if self.game_over {
            return;
        }

        let head = self.snake.front().unwrap();
        // Define possible moves and their resulting positions
        let possible_moves = [
            (Direction::Up, Point { x: head.x, y: head.y - 1 }),
            (Direction::Down, Point { x: head.x, y: head.y + 1 }),
            (Direction::Left, Point { x: head.x - 1, y: head.y }),
            (Direction::Right, Point { x: head.x + 1, y: head.y }),
        ];

        // Simple AI: Choose the direction that gets closer to the food
        let mut best_move = None;
        let mut min_distance = f64::MAX;

        // Evaluate each possible move
        for (dir, point) in possible_moves.iter() {
            // Check if move is valid (within bounds and doesn't hit snake)
            if point.x >= 0
                && point.x < self.width
                && point.y >= 0
                && point.y < self.height
                && !self.snake.contains(point)
            {
                // Calculate Euclidean distance to food
                let distance = (((point.x - self.food.x).pow(2) + (point.y - self.food.y).pow(2)) as f64).sqrt();
                // Update best move if this is the closest to food so far
                if distance < min_distance {
                    min_distance = distance;
                    best_move = Some(*dir);
                }
            }
        }

        // Update direction if a valid move was found
        if let Some(dir) = best_move {
            self.direction = dir;
        }
    }
} 