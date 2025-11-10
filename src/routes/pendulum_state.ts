export type PendulumState = {
    bobs: { theta: number; position: { x: number; y: number }; mass: number; lengthRod: number; omega: number }[];
};