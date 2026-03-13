Ensure S implements Default, per Mealy machine which requires an initial state

Replace "event" with input 
"Action<M>" with "Transaction<M>" or Output? (?)

Keep Mutation? MutOp? The latter seems TOO rust-y,

I think we should separate the operations and transactions from "after" effedcts




enum WrappedInput<InA> {
    Input{a: InA},
    Undo,
    Redo
}

enum WrappedOutput<OutA, InA> {
    Accepted(Transaction)
    Cancelled(Input),   // Cancelled(why: Vec(str?))
    Undid(Transaction),
    Redid(Transaction),
}

Output {
    Transaction<M>
    Triggers? Flags?
    Should it be a Result?
}

enum IResult {
    Accepted,
    Cancelled, // wrap string? enum?


}

Output {

    Undid<Transaction<M>>,
    Redid<Transaction<M>>
}


Outcome::Committed(record)

Outcome::InvalidInput(info)

Outcome::Disallowed(info)

Outcome::NeedsChoice(info) optionally

Outcome::Aborted(info)

with EngineError outside that.