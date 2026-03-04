from crewai import Agent, Task, Crew, Process
import os

def main():
    # Needs OPENAI_API_KEY env for default LLM
    researcher = Agent(
        role='Researcher',
        goal='Research new AI tools',
        backstory='You are an AI research assistant.',
        verbose=True,
        allow_delegation=False
    )
    
    task1 = Task(description='Investigate the latest AI agents.', expected_output='A summary of AI agents.', agent=researcher)
    
    crew = Crew(
        agents=[researcher],
        tasks=[task1],
        verbose=True,
        process=Process.sequential
    )
    
    result = crew.kickoff()
    print("Crew work completed.")
    print(result)

if __name__ == "__main__":
    main()
