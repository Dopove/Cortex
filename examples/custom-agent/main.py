# Custom pure-python agent example
import time
import os

class SimpleAgent:
    def __init__(self, system_prompt):
        self.prompt = system_prompt
        self.memory = []
        
    def respond(self, input_text):
        print(f"Agent interpreting: {input_text}")
        self.memory.append(input_text)
        # Mocking 20ms thinking overhead
        time.sleep(0.02)
        return "I am a custom vanilla Python agent. My latency is extremely low!"

def main():
    agent = SimpleAgent("You are a lightweight custom agent")
    response = agent.respond("Hello Cortex!")
    print(f"Agent replied: {response}")

if __name__ == "__main__":
    main()
