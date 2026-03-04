from langgraph.graph import Graph

def simple_node(state: str) -> str:
    return state + " processed"

def main():
    workflow = Graph()
    workflow.add_node("node1", simple_node)
    workflow.set_entry_point("node1")
    workflow.set_finish_point("node1")
    
    app = workflow.compile()
    
    # We do a basic state invocation without hitting an LLM
    result = app.invoke("Initial state")
    print("LangGraph result:", result)

if __name__ == "__main__":
    main()
