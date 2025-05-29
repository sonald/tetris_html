import ctypes
import numpy as np
import gymnasium as gym
from gymnasium import spaces
import platform
import os

# Define the GameState structure for ctypes
class GameState(ctypes.Structure):
    _fields_ = [
        ("score", ctypes.c_int32),
        ("lost", ctypes.c_bool),
        ("width", ctypes.c_uint32),
        ("height", ctypes.c_uint32),
    ]

class TetrisEnv(gym.Env):
    metadata = {'render_modes': ['human', 'ansi'], 'render_fps': 4}

    def __init__(self, lib_path: str = None, width: int = 10, height: int = 20, render_mode: str = None):
        super().__init__()

        self.width = width
        self.height = height
        self.game_ptr = None # Pointer to the Rust Tetris object
        self.render_mode = render_mode

        if lib_path is None:
            # Determine default library path based on OS and architecture
            base_path = os.path.join(os.path.dirname(__file__), "target", "debug")
            lib_name = ""
            system = platform.system()
            if system == "Linux":
                lib_name = "libtetris_core.so"
            elif system == "Windows":
                lib_name = "tetris_core.dll"
            elif system == "Darwin": # macOS
                lib_name = "libtetris_core.dylib"
            else:
                raise OSError(f"Unsupported OS: {system}. Please provide lib_path manually.")
            self.lib_path = os.path.join(base_path, lib_name)
        else:
            self.lib_path = lib_path
        
        if not os.path.exists(self.lib_path):
            raise OSError(f"Tetris library not found at {self.lib_path}. "
                          "Ensure the Rust code is compiled and the path is correct.")

        self.rust_lib = ctypes.CDLL(self.lib_path)
        self._define_ffi_argtypes()

        self.game_ptr = self.rust_lib.tetris_create(ctypes.c_uint32(self.width), ctypes.c_uint32(self.height))
        if not self.game_ptr:
            raise MemoryError("Failed to create Tetris game instance from Rust library.")

        # Define action and observation spaces
        # 0:left, 1:right, 2:rotate, 3:drop, 4:tick (move down)
        self.action_space = spaces.Discrete(5)
        self.observation_space = spaces.Box(
            low=0, high=1, shape=(self.height, self.width), dtype=np.uint8
        )

        # Initial reset to set up state
        # obs, info = self.reset() 
        # print(f"Initial state: obs shape {obs.shape}, info {info}")


    def _define_ffi_argtypes(self):
        # tetris_create(width: u32, height: u32) -> *mut Tetris
        self.rust_lib.tetris_create.restype = ctypes.c_void_p # Represents *mut Tetris
        self.rust_lib.tetris_create.argtypes = [ctypes.c_uint32, ctypes.c_uint32]

        # tetris_destroy(ptr: *mut Tetris)
        self.rust_lib.tetris_destroy.restype = None
        self.rust_lib.tetris_destroy.argtypes = [ctypes.c_void_p]

        # tetris_reset(ptr: *mut Tetris)
        self.rust_lib.tetris_reset.restype = None
        self.rust_lib.tetris_reset.argtypes = [ctypes.c_void_p]

        # tetris_step(ptr: *mut Tetris, action: u32) -> GameState
        self.rust_lib.tetris_step.restype = GameState
        self.rust_lib.tetris_step.argtypes = [ctypes.c_void_p, ctypes.c_uint32]

        # tetris_get_board(ptr: *const Tetris, out_board_buffer: *mut u8)
        self.rust_lib.tetris_get_board.restype = None
        self.rust_lib.tetris_get_board.argtypes = [
            ctypes.c_void_p,
            ctypes.POINTER(ctypes.c_uint8), # *mut u8
        ]

        # tetris_get_game_state(ptr: *const Tetris) -> GameState
        self.rust_lib.tetris_get_game_state.restype = GameState
        self.rust_lib.tetris_get_game_state.argtypes = [ctypes.c_void_p]

    def _get_obs(self) -> np.ndarray:
        if not self.game_ptr:
            # Return a zeroed observation if game_ptr is None, e.g. after close()
            return np.zeros(shape=(self.height, self.width), dtype=np.uint8)
            
        board_size = self.height * self.width
        # Create a flat buffer for the board data
        board_buffer = (ctypes.c_uint8 * board_size)()
        
        self.rust_lib.tetris_get_board(
            self.game_ptr, 
            board_buffer # ctypes automatically converts array to pointer
        )
        
        # Convert the ctypes array to a NumPy array and reshape
        obs = np.ctypeslib.as_array(board_buffer).reshape(self.height, self.width)
        return obs.copy() # Return a copy to avoid issues with buffer reuse

    def _get_info(self) -> dict:
        if not self.game_ptr:
            return {"score": 0, "lost": True, "width": self.width, "height": self.height}

        state_struct = self.rust_lib.tetris_get_game_state(self.game_ptr)
        return {
            "score": state_struct.score,
            "lost": state_struct.lost,
            "width": state_struct.width,
            "height": state_struct.height,
        }

    def reset(self, seed=None, options=None) -> tuple[np.ndarray, dict]:
        super().reset(seed=seed) # Handles seed if necessary for future reproducibility

        if not self.game_ptr:
             # This case should ideally not happen if constructor succeeded,
             # but as a safeguard if reset is called after close.
            raise ConnectionError("Rust game instance not available. Cannot reset.")

        self.rust_lib.tetris_reset(self.game_ptr)
        
        observation = self._get_obs()
        info = self._get_info()
        
        if self.render_mode == "human":
            self.render()
            
        return observation, info

    def step(self, action: int) -> tuple[np.ndarray, float, bool, bool, dict]:
        if not self.game_ptr:
            raise ConnectionError("Rust game instance not available. Cannot step.")

        # Get current score for reward calculation
        prev_game_state = self.rust_lib.tetris_get_game_state(self.game_ptr)
        
        # Perform the action
        new_game_state_struct = self.rust_lib.tetris_step(self.game_ptr, ctypes.c_uint32(action))
        
        observation = self._get_obs()
        terminated = new_game_state_struct.lost
        
        # Reward calculation
        # Basic reward: score difference. Penalty for losing. Small penalty per step.
        reward = float(new_game_state_struct.score - prev_game_state.score)
        
        # Check if score is based on lines cleared (e.g. 1 point per line)
        # If lines_cleared = new_game_state.score - prev_game_state.score,
        # then reward can be lines_cleared ** 2 or similar for super-linear reward.
        # For now, using simple score difference.
        # Example: if a line clear gives 1 point, reward is 1. If 4 lines give 4 points, reward is 4.
        # If one wants to reward based on lines_cleared^2, and score is 1 per line:
        # lines_cleared = new_game_state.score - prev_game_state.score
        # if lines_cleared > 0:
        #    reward = float(lines_cleared ** 2) # e.g. 1, 4, 9, 16 for 1,2,3,4 lines
        # else:
        #    reward = 0.0 # No lines cleared
        
        reward -= 0.01 # Small penalty per step to encourage efficiency

        if terminated:
            reward -= 100.0 # Large penalty for losing

        truncated = False # Tetris typically doesn't truncate early unless a step limit is imposed
        
        info = {
            "score": new_game_state_struct.score,
            "lost": new_game_state_struct.lost,
            "width": new_game_state_struct.width,
            "height": new_game_state_struct.height,
        }

        if self.render_mode == "human":
            self.render()
            
        return observation, reward, terminated, truncated, info

    def render(self):
        if not self.game_ptr and self.render_mode in ["human", "ansi"]:
             print("No game instance to render.")
             return

        obs = self._get_obs()
        if self.render_mode == 'human':
            # Simple console print, replace # with block, . with empty
            print("\033[H\033[J", end="") # Clear screen
            for r in range(self.height):
                row_str = "".join(["⬜️" if obs[r, c] == 1 else "⬛️" for c in range(self.width)])
                print(row_str)
            info = self._get_info()
            print(f"Score: {info['score']} | Lost: {info['lost']}")
            print("-" * (self.width * 2)) # Separator line
        elif self.render_mode == 'ansi':
            # Could return a string representation for ANSI
            info = self._get_info()
            board_str = "\n".join("".join(["#" if obs[r, c] == 1 else "." for c in range(self.width)]) for r in range(self.height))
            return f"{board_str}\nScore: {info['score']} | Lost: {info['lost']}"


    def close(self):
        if self.game_ptr:
            self.rust_lib.tetris_destroy(self.game_ptr)
            self.game_ptr = None
            # print("Tetris game instance destroyed.") # Optional: for debugging

    def __del__(self):
        self.close()

if __name__ == '__main__':
    # Example usage:
    # Ensure the .so/.dll/.dylib is in target/debug/ relative to this script, or provide full path.
    # For example, if tetris_env.py is in the root of your Rust project, and the lib is in target/debug.
    
    # Determine the path to the library dynamically based on script location
    # Assumes the script is in the root of the project.
    script_dir = os.path.dirname(os.path.abspath(__file__))
    default_lib_path = None
    
    # Construct path assuming 'target/debug/' relative to the script's directory
    # This is a common setup if the Python script is in the root of the Rust project
    rust_project_root = script_dir # Assuming script is at project root
    
    # More robust: try to find target/debug relative to current working directory if it's the project root
    # Or expect user to set an environment variable or pass it.
    # For now, let's assume target/debug is in the same directory as the script or one level up if script is in a 'scripts' folder
    
    # Try a common path structure: <project_root>/target/debug/libtetris_core.so
    # If tetris_env.py is at project_root:
    path_to_target_debug = os.path.join(rust_project_root, "target", "debug")

    system = platform.system()
    if system == "Linux":
        default_lib_path = os.path.join(path_to_target_debug, "libtetris_core.so")
    elif system == "Windows":
        default_lib_path = os.path.join(path_to_target_debug, "tetris_core.dll")
    elif system == "Darwin": # macOS
        default_lib_path = os.path.join(path_to_target_debug, "libtetris_core.dylib")

    print(f"Attempting to load library from: {default_lib_path}")

    if default_lib_path and os.path.exists(default_lib_path):
        env = TetrisEnv(lib_path=default_lib_path, render_mode='human')
        
        obs, info = env.reset()
        print(f"Initial Observation:\n{obs}")
        print(f"Initial Info: {info}")

        terminated = False
        total_reward = 0
        steps = 0
        try:
            for _ in range(1000): # Run for a number of steps
                action = env.action_space.sample() # Sample a random action
                obs, reward, terminated, truncated, info = env.step(action)
                total_reward += reward
                steps +=1
                
                # env.render() # Already called in step if render_mode='human'
                import time
                time.sleep(0.1) # Slow down for human viewing

                if terminated or truncated:
                    print(f"Episode finished after {steps} steps.")
                    print(f"Final Observation:\n{obs}")
                    print(f"Final Info: {info}")
                    print(f"Total Reward: {total_reward}")
                    break
        except KeyboardInterrupt:
            print("Simulation interrupted by user.")
        finally:
            env.close()
    else:
        print(f"Library not found at default path: {default_lib_path}")
        print("Please compile the Rust library and ensure the path is correct.")
        print("You might need to run `cargo build` in the Rust project directory.")
