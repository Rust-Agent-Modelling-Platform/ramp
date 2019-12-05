package pl.edu.agh.mason;

import sim.engine.SimState;

import java.util.ArrayList;

public class Simulation {
    public static void main(String[] args) {
        if (args.length != 3){
            System.out.println("Program parameters: <genotype_dim> <islands> <turns>");
            System.exit(1);
        }
        AgentConfig config = new AgentConfig(
                Integer.parseInt(args[0]),
                100,
                true,
                0.01,
                40,
                0.5,
                40,
                -5.12,
                5.12
        );

        long time = System.currentTimeMillis();

        int islandNum = Integer.parseInt(args[1]);
        int turns = Integer.parseInt(args[2]);
        ArrayList<Island> islands = new ArrayList<>();

        // Create islands
        for (int i = 0; i < islandNum; i++) {
            islands.add(new Island(time, i, config));
        }

        //Provide islands with reference to all other islands
        for (Island island : islands) {
            island.setIslandRef(islands);
        }

        // Start islands
        for(SimState island : islands) {
            island.start();
        }

        //Run simulation
        int counter = 0;
        while(counter < turns) {
            for(SimState island : islands) {
                island.schedule.step(island);
            }
            counter++;
        }

        // Finish
        for(SimState island : islands) {
            island.finish();
        }

        // Exit
        System.exit(0);
    }
}
