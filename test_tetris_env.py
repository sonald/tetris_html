import unittest
import numpy as np
import gymnasium as gym
from gymnasium.utils.env_checker import check_env
from tetris_env import TetrisEnv # Assuming tetris_env.py is in the same directory
import os
import platform

class TestTetrisEnv(unittest.TestCase):
    def setUp(self):
        """Set up the test environment before each test."""
        lib_name = None
        system = platform.system()

        if system == "Linux":
            lib_name = "libtetris_core.so"
        elif system == "Windows":
            lib_name = "tetris_core.dll"
        elif system == "Darwin": # macOS
            lib_name = "libtetris_core.dylib"
        else:
            raise OSError(f"Unsupported OS for testing: {system}")

        # Assuming tests are run from the project root where target/debug is a subdirectory
        # This path should align with where `cargo build` places the dynamic library.
        # If tetris_env.py is also in the root, this path structure is consistent with TetrisEnv's default.
        self.lib_path = os.path.join(os.getcwd(), "target", "debug", lib_name)
        
        if not os.path.exists(self.lib_path):
            # Attempt to build the library if it's missing, useful for CI/first run
            # This assumes `cargo` is in PATH and the current working directory is the project root.
            print(f"Library not found at {self.lib_path}. Attempting to build Rust library...")
            try:
                import subprocess
                # Ensure we are in the project root for cargo build
                project_root = os.getcwd() # Or determine more robustly if needed
                subprocess.run(["cargo", "build"], check=True, cwd=project_root, capture_output=True)
                print("Rust library built successfully.")
                if not os.path.exists(self.lib_path): # Check again after build
                     raise FileNotFoundError(f"Library still not found at {self.lib_path} after attempting build.")
            except FileNotFoundError as e: # cargo not found
                print(f"Error: 'cargo' command not found. Please ensure Rust is installed and in PATH. {e}")
                raise e
            except subprocess.CalledProcessError as e:
                print(f"Error building Rust library with Cargo: {e}")
                print(f"Stdout: {e.stdout.decode()}")
                print(f"Stderr: {e.stderr.decode()}")
                raise e
            except Exception as e:
                print(f"An unexpected error occurred during library build attempt: {e}")
                raise e


        self.env = TetrisEnv(lib_path=self.lib_path, width=10, height=20)

    def tearDown(self):
        """Clean up the test environment after each test."""
        self.env.close()

    def test_initialization(self):
        """Test if the environment initializes correctly."""
        self.assertIsNotNone(self.env)
        self.assertIsInstance(self.env.observation_space, gym.spaces.Box)
        self.assertEqual(self.env.observation_space.shape, (20, 10))
        self.assertEqual(self.env.observation_space.dtype, np.uint8)
        self.assertIsInstance(self.env.action_space, gym.spaces.Discrete)
        self.assertEqual(self.env.action_space.n, 5)

    def test_reset(self):
        """Test the reset method."""
        obs, info = self.env.reset()
        self.assertIsInstance(obs, np.ndarray)
        self.assertEqual(obs.shape, self.env.observation_space.shape)
        self.assertEqual(obs.dtype, self.env.observation_space.dtype)
        self.assertIsInstance(info, dict)
        self.assertIn("score", info)
        self.assertIn("lost", info)
        self.assertEqual(info["score"], 0) # Assuming score resets to 0
        self.assertEqual(info["lost"], False) # Assuming not lost on reset

    def test_step(self):
        """Test the step method with a few actions."""
        self.env.reset()
        for action in range(self.env.action_space.n): # Test all possible actions
            obs, reward, terminated, truncated, info = self.env.step(action)
            
            self.assertIsInstance(obs, np.ndarray)
            self.assertEqual(obs.shape, self.env.observation_space.shape)
            self.assertEqual(obs.dtype, self.env.observation_space.dtype)
            
            self.assertIsInstance(reward, float)
            self.assertIsInstance(terminated, bool)
            self.assertIsInstance(truncated, bool)
            self.assertIsInstance(info, dict)
            self.assertIn("score", info)
            self.assertIn("lost", info)
            
            if terminated: # If game ends early, no need to continue testing all actions
                break

    def test_random_rollout(self):
        """Test a short rollout with random actions."""
        obs, info = self.env.reset()
        max_steps = 25 # Increased steps to have higher chance of game events
        
        for i in range(max_steps):
            action = self.env.action_space.sample()
            obs, reward, terminated, truncated, info = self.env.step(action)

            self.assertIsInstance(obs, np.ndarray)
            self.assertEqual(obs.shape, self.env.observation_space.shape)
            self.assertEqual(obs.dtype, self.env.observation_space.dtype)
            self.assertIsInstance(reward, float)
            self.assertIsInstance(terminated, bool)
            self.assertIsInstance(truncated, bool)
            self.assertIsInstance(info, dict)

            if terminated or truncated:
                # print(f"Rollout terminated/truncated at step {i+1}")
                # It's valid for an episode to end, so we reset and can continue
                # or simply break if we just want to ensure steps work until termination.
                # For this test, breaking is fine.
                break
        # self.assertTrue(i < max_steps -1 or terminated or truncated, "Rollout did not terminate or finish all steps")


    def test_gymnasium_check_env(self):
        """Test compliance with Gymnasium API using check_env."""
        # The environment instance passed to check_env should be the base environment,
        # not wrapped, if wrappers are ever added. self.env.unwrapped is good practice.
        # skip_render_check=True because console rendering is hard to check automatically
        # and might require specific terminal capabilities.
        try:
            check_env(self.env.unwrapped, skip_render_check=True)
        except Exception as e:
            self.fail(f"check_env failed: {e}")

    def test_game_over_reward_check(self):
        """Test that reward is significantly negative upon game over."""
        self.env.reset()
        terminated = False
        total_reward_before_loss = 0
        last_reward = 0

        # Try to force a game over by repeatedly dropping pieces.
        # This is heuristic and depends on the game logic.
        # Action 3 is speed_up (drop), Action 4 is tick (move down)
        # We'll primarily use speed_up and occasional ticks.
        for i in range(100): # Max steps to try to trigger game over
            action = 3 # speed_up (drop piece to bottom)
            if i % 5 == 0 and i > 0: # Occasionally just tick to let things settle or new piece appear
                action = 4 # tick

            obs, reward, terminated, truncated, info = self.env.step(action)
            last_reward = reward
            if not terminated:
                total_reward_before_loss += reward
            
            if terminated:
                # print(f"Game over detected at step {i+1}. Last reward: {last_reward}, Info: {info}")
                # Expecting a significant penalty for losing.
                # The specific value (-100.0) comes from TetrisEnv's reward logic.
                self.assertTrue(reward <= -100.0, f"Reward on game over should be <= -100, got {reward}")
                break
        
        self.assertTrue(terminated, "Game did not terminate within the given steps for game_over test.")

if __name__ == '__main__':
    unittest.main()
```
