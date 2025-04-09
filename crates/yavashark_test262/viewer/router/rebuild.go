package router

import "os/exec"

func rebuild() error {
	cmd := exec.Command("cargo", "build", "--release")

	//TODO: we should pass stderr and stdout to the client

	//stdout, err := cmd.StdoutPipe()
	//if err != nil {
	//	return err
	//}
	//
	//stdout.Read()

	return cmd.Run()
}
