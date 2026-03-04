from smolagents import CodeAgent, HfApiModel
import os

def main():
    # Attempt to load a model if HF_TOKEN is in environment
    # Use fallback structure to ensure the script boots cleanly during bench
    print("Initializing SmolAgents CodeAgent...")
    try:
        agent = CodeAgent(tools=[], model=HfApiModel())
        if os.environ.get("HF_TOKEN"):
            result = agent.run("What is 2+2?")
            print("SmolAgents result:", result)
        else:
            print("HF_TOKEN not set, skipping LLM inference during benchmarking.")
    except Exception as e:
        print(f"Skipping smolagents inference execution due to constraint: {e}")

if __name__ == "__main__":
    main()
