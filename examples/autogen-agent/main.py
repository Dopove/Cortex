from autogen import ConversableAgent
import os

def main():
    # Use dummy API key for build checks, user can pass actual via ENV
    api_key = os.environ.get("OPENAI_API_KEY", "dummy-key")
    agent = ConversableAgent(
        "chatbot",
        llm_config={"config_list": [{"model": "gpt-4", "api_key": api_key}]},
        code_execution_config=False,
    )
    
    reply = agent.generate_reply(messages=[{"content": "Tell me a joke.", "role": "user"}])
    print(reply)

if __name__ == "__main__":
    main()
