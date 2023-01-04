use actix::Message;

#[derive(Message, Debug, PartialEq, Clone)]
#[rtype(result = "()")]
pub struct Activity;

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Connect;

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Close;

#[derive(Message, Debug, PartialEq, Clone)]
#[rtype(result = "()")]
pub struct Job;
