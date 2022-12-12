use actix::Message;

#[derive(Message, Debug, PartialEq, Clone)]
#[rtype(result = "()")]
pub struct Activity;

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Ping;

#[derive(Message, Debug, PartialEq, Clone)]
#[rtype(result = "()")]
pub struct Job;
