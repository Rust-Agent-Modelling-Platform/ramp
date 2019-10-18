package pl.edu.agh.mason;

import sim.engine.SimState;
import sim.engine.Stoppable;

import java.util.*;

public class Island extends SimState {

    public Island(long seed, int i, AgentConfig config) {
        super(seed);
        this.islandId = i;
        this.config = config;
    }

    public void setIslandRef(ArrayList<Island> islandList) {
        this.islandList = islandList;
    }


    private int agentNum = 50;
    private AgentConfig config;

    int islandId;
    ArrayList<Island> islandList;

    Map<UUID, Agent> agents = new HashMap<>();
    Map<UUID, Stoppable> stopHandles = new HashMap<>();

    List<UUID> dead = new ArrayList<>();
    List<UUID> procreating = new ArrayList<>();
    List<UUID> meeting = new ArrayList<>();
    List<UUID> migrating = new ArrayList<>();

    ArrayList<Island> getIslandList() {
        return this.islandList;
    }

    int getIslandId() {
        return this.islandId;
    }


    public void start() {
        super.start();

        for (int i = 1; i <= agentNum; i++) {
            UUID id = UUID.randomUUID();
            Agent agent = new Agent(id, config);
            Stoppable stopHandle = schedule.scheduleRepeating(agent);

            this.stopHandles.put(id, stopHandle);
            this.agents.put(id, agent);
        }

        ActionResolutionAgent resolutionAgent = new ActionResolutionAgent(0);
        System.out.println("Created resolution agent");
        schedule.addAfter(resolutionAgent);
    }

    public static void main(String[] args) {
        doLoop(Island.class, args);
        System.exit(0);
    }

}
