package pl.edu.agh.mason;

public class AgentConfig {

    int genotypeDim;
    int initialEnergy;
    boolean minimum;
    double mutationRate;
    int procreationProb;
    double procreationPenalty;
    int meetingPenalty;
    double lowerBound;
    double upperBound;


    public AgentConfig(int genotypeDim,
                       int initialEnergy,
                       boolean minimum,
                       double mutationRate,
                       int procreationProb,
                       double procreationPenalty,
                       int meetingPenalty,
                       double lowerBound,
                       double upperBound)
    {
        this.genotypeDim = genotypeDim;
        this.initialEnergy = initialEnergy;
        this.minimum = minimum;
        this.mutationRate = mutationRate;
        this.procreationProb = procreationProb;
        this.procreationPenalty = procreationPenalty;
        this.meetingPenalty = meetingPenalty;
        this.lowerBound = lowerBound;
        this.upperBound = upperBound;

    }

}



