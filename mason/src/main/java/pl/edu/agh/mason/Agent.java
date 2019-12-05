package pl.edu.agh.mason;

import sim.engine.SimState;
import sim.engine.Steppable;

import java.util.Arrays;
import java.util.Random;
import java.util.UUID;
import java.util.concurrent.ThreadLocalRandom;

import static java.lang.Math.PI;

public class Agent implements Steppable {

    private UUID id;
    private AgentConfig config;
    private double[] genotype;
    private double fitness;
    private int energy;

    public Agent(UUID id, AgentConfig config) {
        this.id = id;
        this.config = config;
        this.genotype = generateGenotype(config.genotypeDim, config.lowerBound, config.upperBound);
        this.fitness = calculateFitness(genotype, config.minimum);
        this.energy = config.initialEnergy;
    }

    @Override
    public void step(SimState state) {
        Island island = (Island) state;
        getAction(island);
    }

    public void getAction(Island island) {
        int prob = ThreadLocalRandom.current().nextInt(0, 100);
        if (this.energy <= 0) {
            island.dead.add(this.id);
        } else if (prob == 1) {
            island.migrating.add(this.id);
        } else if (this.energy <= 90) {
            island.meeting.add(this.id);
        } else if (prob >= this.config.procreationProb) {
            island.procreating.add(this.id);
        } else {
            island.meeting.add(this.id);
        }
    }

    Agent procreate(Agent partner) {
        Agent newAgent = new Agent(UUID.randomUUID(), this.config);
        newAgent.setGenotype(crossover(this.genotype, partner.genotype));
        newAgent.mutate();

        this.energy = (int) Math.floor(this.energy * (1.0 - config.procreationPenalty));
        partner.energy = (int) Math.floor(partner.energy * (1.0 - config.procreationPenalty));

        return newAgent;
    }

    void meet(Agent partner) {
        if (this.fitness > partner.fitness) {
            this.energy += config.meetingPenalty;
            partner.energy -= config.meetingPenalty;
        } else {
            this.energy -= config.meetingPenalty;
            partner.energy += config.meetingPenalty;
        }
    }

    //-------------------------------------------------------------------------------------------------

    private void mutate() {
        double leftBound = this.config.lowerBound / 10.0;
        double rightBound = this.config.upperBound / 10.0;

        for (int i = 0; i < genotype.length; i++) {
            Random r = new Random();
            double val = (0.0 + (1.0 - 0.0) * r.nextDouble());
            if (val <= this.config.mutationRate) {
                Random r2 = new Random();
                double delta = (leftBound + (rightBound - leftBound) * r2.nextDouble());
                this.genotype[i] += delta;
            }
        }
    }

    private double[] crossover(double[] genotype1, double[] genotype2) {
        double[] newGenotype = new double[genotype1.length];
        int divisionPoint = ThreadLocalRandom.current().nextInt(0, genotype1.length);
        for (int i = 0; i < divisionPoint; i++) {
            newGenotype[i] = genotype1[i];
        }
        for (int i = divisionPoint; i < genotype2.length; i++) {
            newGenotype[i] = genotype2[i];
        }
        return newGenotype;
    }


    //-------------------------------------------------------------------------------------------------
    private void setGenotype(double[] genotype) {
        this.genotype = genotype;
        this.fitness = calculateFitness(genotype, this.config.minimum);
    }

    public UUID getId() {
        return this.id;
    }
    //-------------------------------------------------------------------------------------------------

    private double[] generateGenotype(int dim, double lowBound, double upBound) {
        double[] genotype = new double[dim];
        for (int i = 0; i < genotype.length; i++) {
            Random r = new Random();
            double randomValue = lowBound + (upBound - lowBound) * r.nextDouble();
            genotype[i] = randomValue;
        }
        return genotype;
    }

    private double calculateFitness(double[] genotype, boolean minimum) {
        double fitness;
        double sum = 0;
        double A = 10.0;
        for (double gene : genotype) {
            sum += Math.pow(gene, 2.0) - A * Math.cos((2.0 * PI * gene));
        }
        fitness = A * this.config.genotypeDim + sum;
        return minimum ? -fitness : fitness;
    }

    public void print() {
        System.out.println(".....................................................");
        System.out.println("BEST AGENT:");
        System.out.println("Agent id: " + this.id.toString().substring(0, 8));
        System.out.println("Energy: " + this.energy);
        System.out.println(Colors.ANSI_BLUE + "Fitness: " + this.fitness + Colors.ANSI_RESET);
        System.out.println("Genotype: " + Arrays.toString(this.genotype));
        System.out.println(".......................................................");
    }

    public double getFitness() {
        return this.fitness;
    }

}


