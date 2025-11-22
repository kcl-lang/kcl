schema Job:
    name: str

    createResource: () -> CronJob = lambda {
        CronJob {
            name = name
        }
    }
    getName: () -> str = lambda {
        name
    }

schema CronJob:
    name?: str

myJob = Job {
    name = "myJob"
}
myCronJob = myJob.createResource()
name = myJob.getName()
