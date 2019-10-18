package pl.edu.agh.mason;

import sim.engine.SimState;
import sim.engine.Steppable;
import sim.engine.Stoppable;

import java.util.*;
import java.util.concurrent.ThreadLocalRandom;

public class ActionResolutionAgent implements Steppable {
    private int id;

    ActionResolutionAgent(int id) {
        this.id = id;
    }

    @Override
    public void step(SimState state) {
        Island island = (Island) state;

        System.out.println("Dead agents: " + island.dead.size());
        resolveDeads(island);
        resolveMigrations(island);
        resolveProcreation(island);
        resolveMeetings(island);
        printBestAgent(island.agents);

        System.out.println("Agents in system: " + island.agents.size());
        System.out.println(Colors.ANSI_GREEN +
                "========================= END OF TURN " + (island.schedule.getTime())
                + " ON ISLAND " + island.islandId +
                " =========================" + Colors.ANSI_RESET);
    }

    private void resolveDeads(Island island) {
        for (UUID deadId : island.dead) {
            island.stopHandles.remove(deadId).stop();
            island.agents.remove(deadId);
        }
        island.dead.clear();
    }

    private void resolveProcreation(Island island) {
        System.out.println(">>> Resolving procreating agents");
        List<UUID> procreating = island.procreating;
        if (procreating.size() % 2 == 1) {
            procreating.remove(0);
        }
        System.out.println(procreating.size());
        Iterator<UUID> itr = procreating.iterator();
        while (itr.hasNext()) {
            Agent agent1 = island.agents.get(itr.next());
            Agent agent2 = island.agents.get(itr.next());

            Agent newAgent = agent1.procreate(agent2);

            island.agents.put(newAgent.getId(), newAgent);
            Stoppable stopHandle = island.schedule.scheduleRepeating(newAgent);
            island.stopHandles.put(newAgent.getId(), stopHandle);
        }
        island.procreating.clear();
    }

    private void resolveMeetings(Island island) {
        System.out.println(">>> Resolving meetings");
        List<UUID> meeting = island.meeting;
        if (meeting.size() % 2 == 1) {
            meeting.remove(0);
        }
        System.out.println(meeting.size());
        Iterator<UUID> itr = meeting.iterator();
        while (itr.hasNext()) {
            Agent agent1 = island.agents.get(itr.next());
            Agent agent2 = island.agents.get(itr.next());
            agent1.meet(agent2);
        }
        island.meeting.clear();
    }

    private void resolveMigrations(Island island) {
        System.out.println(">> Resolving migrations");
        System.out.println(island.migrating.size());
        if (island.migrating.isEmpty()) {
            return;
        }
        ArrayList<Island> islands = island.getIslandList();

        int randomIndex = island.getIslandId();
        while (randomIndex == island.getIslandId()) {
            randomIndex = ThreadLocalRandom.current().nextInt(0, islands.size());
        }
        Island targetIsland = islands.get(randomIndex);

        for (int i = 0; i < island.migrating.size(); i++) {
            UUID migratingAgentId = island.migrating.get(i);
            Agent migratingAgent = island.agents.get(migratingAgentId);

            island.stopHandles.remove(migratingAgentId).stop();

            targetIsland.agents.put(migratingAgentId, migratingAgent);
            Stoppable stopHandle = targetIsland.schedule.scheduleRepeating(migratingAgent);
            targetIsland.stopHandles.put(migratingAgentId, stopHandle);

            island.agents.remove(migratingAgentId);
        }
        island.migrating.clear();
    }


    //---------------------------------------------------------------------------------------
    private void printBestAgent(Map<UUID, Agent> agents) {
        Agent bestAgent = (Agent) agents.values().toArray()[0];
        for (Agent agent : agents.values()) {
            if (agent.getFitness() >= bestAgent.getFitness()) {
                bestAgent = agent;
            }
        }
        bestAgent.print();
    }
}
