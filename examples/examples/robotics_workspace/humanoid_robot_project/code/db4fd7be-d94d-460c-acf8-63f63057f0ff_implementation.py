import numpy as np

class RobotSimulation:
    """
    Minimal robot simulation environment with basic physics and control
    """
    
    def __init__(self, dt=0.01, gravity=-9.81):
        self.dt = dt  # simulation timestep
        self.gravity = gravity
        self.time = 0.0
        self.robot_state = {
            'position': np.array([0.0, 0.0, 0.0]),  # [x, y, z]
            'velocity': np.array([0.0, 0.0, 0.0]),  # [vx, vy, vz]
            'acceleration': np.array([0.0, 0.0, gravity])  # [ax, ay, az]
        }
    
    def apply_force(self, force):
        """Apply external force to robot (e.g., thrusters, motors)"""
        # F = ma -> a = F/m (assuming unit mass)
        self.robot_state['acceleration'] = np.array([0.0, 0.0, self.gravity]) + force
    
    def step(self):
        """Advance simulation by one timestep using Euler integration"""
        # Update velocity: v = v0 + a*dt
        self.robot_state['velocity'] += self.robot_state['acceleration'] * self.dt
        
        # Update position: x = x0 + v*dt
        self.robot_state['position'] += self.robot_state['velocity'] * self.dt
        
        self.time += self.dt
        return self.robot_state.copy()
    
    def reset(self):
        """Reset simulation to initial state"""
        self.time = 0.0
        self.robot_state = {
            'position': np.array([0.0, 0.0, 0.0]),
            'velocity': np.array([0.0, 0.0, 0.0]),
            'acceleration': np.array([0.0, 0.0, self.gravity])
        }

# Example usage
if __name__ == "__main__":
    sim = RobotSimulation(dt=0.01)
    
    # Apply upward force (like a thruster)
    sim.apply_force(np.array([0.0, 0.0, 15.0]))
    
    # Run simulation for 100 steps
    for i in range(100):
        state = sim.step()
        if i % 20 == 0:
            print(f"Time: {sim.time:.2f}s, Position: {state['position']}")