#!/usr/bin/env python3
"""
Main script for dependency test project.
This demonstrates the Agent class usage and dependency patterns.
"""

import asyncio
import logging
from core.agent import Agent, AgentConfig, AgentType, create_agent
from core.messaging import Message, Priority
from utils.logger import setup_logging


async def main():
    """Main function demonstrating Agent usage."""
    
    # Set up logging
    setup_logging("INFO")
    logger = logging.getLogger(__name__)
    
    logger.info("Starting dependency test project...")
    
    # Create agent configuration
    config = AgentConfig(
        name="dependency_test_agent",
        agent_type=AgentType.BOT,
        debug=True
    )
    
    # Create and initialize agent
    agent = create_agent("specialized", config.__dict__)
    
    try:
        if await agent.initialize():
            logger.info("Agent initialized successfully")
            
            # Create a test message
            message = Message(
                id="msg_001",
                type="test",
                content="Testing dependency patterns",
                timestamp="2024-01-01T00:00:00Z",
                priority=Priority.NORMAL
            )
            
            # Get agent status and metrics
            status = agent.get_status()
            metrics = agent.get_metrics()
            logger.info(f"Agent status: {status.value}")
            logger.info(f"Agent metrics: {metrics.dict()}")
            
            # Perform health check
            health = await agent.health_check()
            logger.info(f"Health check: {health}")
            
            logger.info("Agent demonstration completed successfully")
            
        else:
            logger.error("Failed to initialize agent")
            
    except Exception as e:
        logger.error(f"Error during agent execution: {e}")
        
    finally:
        # Always shutdown cleanly
        await agent.shutdown()
        logger.info("Agent shutdown complete")


if __name__ == "__main__":
    asyncio.run(main()) 